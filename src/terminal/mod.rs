pub mod charsets;
pub mod seq_parser;

use self::charsets::{CODE_PAGE_0, CODE_PAGE_1};
use self::seq_parser::{Action, ClearType, CodePage, LineSize, SeqParser};
use std::{char, f64, mem};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CellStyle {
    attrs: u16,
    fg: u32,
    bg: u32,
}

impl CellStyle {
    fn new() -> CellStyle {
        CellStyle {
            attrs: 0,
            fg: 0,
            bg: 0,
        }
    }

    fn reset(&mut self) {
        self.attrs = 0;
    }

    fn has_short_color(&self) -> bool {
        self.fg < 256 && self.bg < 256
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct CursorState {
    x: i32,
    y: i32,
    style: u8,
    visible: bool,
}

impl CursorState {
    fn new() -> CursorState {
        CursorState {
            x: 0,
            y: 0,
            style: 1,
            visible: true,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct ScreenCell {
    text: char,
    style: CellStyle,
}

impl ScreenCell {
    pub fn set(&mut self, text: char, style: CellStyle) {
        self.text = text;
        self.style = style;
    }
}

struct ScreenBuffer {
    lines: Vec<Vec<ScreenCell>>,
    line_sizes: Vec<LineSize>,
}

impl ScreenBuffer {
    fn new(width: usize, height: usize) -> ScreenBuffer {
        let mut buf = ScreenBuffer {
            lines: Vec::new(),
            line_sizes: Vec::new(),
        };
        buf.clear(width, height, CellStyle::new());
        buf
    }

    fn make_line(width: usize, style: CellStyle) -> Vec<ScreenCell> {
        let mut line: Vec<ScreenCell> = Vec::with_capacity(width);
        for _ in 0..width {
            line.push(ScreenCell { text: ' ', style })
        }
        line
    }

    fn clear(&mut self, width: usize, height: usize, style: CellStyle) {
        self.lines.clear();
        self.line_sizes.clear();

        for _ in 0..height {
            self.lines.push(ScreenBuffer::make_line(width, style));
            self.line_sizes.push(LineSize::default());
        }
    }

    fn resize_lossy(&mut self, width: usize, height: usize, style: CellStyle) {
        let old_lines = self.lines.clone();
        let old_line_sizes = self.line_sizes.clone();

        self.clear(width, height, style);

        for y in 0..old_lines.len().min(height) {
            for x in 0..old_lines[0].len().min(width) {
                self.lines[y][x] = old_lines[y][x];
            }
            self.line_sizes[y] = old_line_sizes[y];
        }
    }
}

struct TerminalState {
    style: CellStyle,
    cursor: CursorState,
    saved_cursor: CursorState,
    track_mouse: bool,
    rainbow: bool,
    is_alt_buffer: bool,
    alt_buffer: ScreenBuffer,
    buffer: ScreenBuffer,
    scroll_margin_top: u32,
    scroll_margin_bottom: u32,
    state_id: u32,
    title: String,
    bell_id: u32,
    bracketed_paste: bool,
    reverse_video: bool,
    charset: u8,
    charsets: Vec<CodePage>,
    last_screen: Vec<ScreenCell>,
}

impl TerminalState {
    fn new(width: usize, height: usize) -> TerminalState {
        TerminalState {
            style: CellStyle::new(),
            cursor: CursorState::new(),
            saved_cursor: CursorState::new(),
            track_mouse: false,
            rainbow: false,
            is_alt_buffer: false,
            alt_buffer: ScreenBuffer::new(width, height),
            buffer: ScreenBuffer::new(width, height),
            scroll_margin_top: 0,
            scroll_margin_bottom: height as u32,
            state_id: 0,
            title: String::new(),
            bell_id: 0,
            bracketed_paste: false,
            reverse_video: false,
            charset: 0,
            charsets: vec![CodePage::USASCII, CodePage::USASCII],
            last_screen: Vec::new(),
        }
    }
}

fn get_rainbow_color(t: f64) -> u32 {
    let r = (t.sin() * 127.0 + 127.0).floor() as u32;
    let g = ((t + 2.0 / 3.0 * f64::consts::PI).sin() * 127.0 + 127.0).floor() as u32;
    let b = ((t + 4.0 / 3.0 * f64::consts::PI).sin() * 127.0 + 127.0).floor() as u32;
    ((r << 16) | (g << 8) | b) + 256
}

pub fn encode_as_code_point(n: u32) -> char {
    // this is unsafe but I don't think C (i.e. ESPTerm) cares either
    unsafe {
        if n >= 0xD800 {
            char::from_u32_unchecked(n + 0x801)
        } else {
            char::from_u32_unchecked(n + 1)
        }
    }
}

pub fn encode_24color(color: u32) -> String {
    let mut result = String::new();
    if color < 256 {
        result.push(encode_as_code_point(color));
    } else {
        let color = color - 256;
        result.push(encode_as_code_point(color & 0xFFF | 0x10000));
        result.push(encode_as_code_point((color >> 12) & 0xFFF));
    }
    result
}

pub struct Terminal {
    pub width: u32,
    pub height: u32,
    parser: SeqParser,
    state: TerminalState,
}

impl Terminal {
    pub fn new(width: u32, height: u32) -> Terminal {
        Terminal {
            width,
            height,
            parser: SeqParser::new(),
            state: TerminalState::new(width as usize, height as usize),
        }
    }

    pub fn is_cursor_hanging(&self) -> bool {
        self.state.cursor.x == self.width as i32
    }

    pub fn set_alt_buffer(&mut self, enabled: bool) {
        if enabled != self.state.is_alt_buffer {
            self.state.is_alt_buffer = enabled;

            mem::swap(&mut self.state.buffer, &mut self.state.alt_buffer);

            if enabled {
                self.clear_screen();
            }
        }
    }

    pub fn clear_screen(&mut self) {
        self.state.buffer.clear(
            self.width as usize,
            self.height as usize,
            self.state.style.clone(),
        );
    }

    pub fn clear_line(&mut self, ln: u32, style: CellStyle) {
        if ln >= self.height {
            return;
        }
        self.state.buffer.lines[ln as usize]
            .iter_mut()
            .for_each(|cell| cell.set(' ', style));
    }

    pub fn clear_line_before(&mut self, ln: u32, col: u32, style: CellStyle) {
        if ln >= self.height {
            return;
        }
        let line = &mut self.state.buffer.lines[ln as usize];
        for x in 0..=col.min(self.width) {
            line[x as usize].set(' ', style);
        }
    }

    pub fn clear_line_after(&mut self, ln: u32, col: u32, style: CellStyle) {
        if ln >= self.height || col >= self.width {
            return;
        }
        let line = &mut self.state.buffer.lines[ln as usize];
        for x in col..self.width {
            line[x as usize].set(' ', style);
        }
    }

    fn copy_line_from_adjacent(&mut self, y: u32, dy: i32) {
        let target = (y as i32) + dy;
        let line;
        if target < 0 || target as u32 >= self.state.scroll_margin_bottom {
            line = ScreenBuffer::make_line(self.width as usize, self.state.style);
        } else {
            line = self.state.buffer.lines[target as usize].clone();
        }
        self.state.buffer.lines[y as usize] = line;
    }

    pub fn scroll(&mut self, amount: i32, with_cursor: bool) {
        if amount >= 0 {
            for y in self.state.scroll_margin_top..self.state.scroll_margin_bottom {
                self.copy_line_from_adjacent(y, amount);
            }
        } else {
            for y in (self.state.scroll_margin_top..self.state.scroll_margin_bottom).rev() {
                self.copy_line_from_adjacent(y, amount);
            }
        }
        if with_cursor {
            self.state.cursor.y -= amount;
            self.clamp_cursor();
        }
    }

    pub fn clamp_cursor(&mut self) {
        self.state.cursor.x = self.state.cursor.x.max(0).min(self.width as i32);
        self.state.cursor.y = self
            .state
            .cursor
            .y
            .max(0)
            .min(self.state.scroll_margin_bottom as i32 - 1);
    }

    pub fn new_line(&mut self) {
        self.state.cursor.y += 1;
        if self.state.cursor.y >= self.state.scroll_margin_bottom as i32 {
            self.scroll(1, true);
        }
    }

    pub fn write_char(&mut self, c: char) {
        if self.state.cursor.x >= self.width as i32 {
            self.state.cursor.x = 0;
            self.new_line();
        }
        let c = if (c as u32) < 128 {
            // check code page
            let code_page = self.state.charsets[self.state.charset as usize];

            macro_rules! code_page_lookup {
                ($cp:expr, $c:expr) => {{
                    if $cp.begin <= ($c as u32) && ($c as u32) <= $cp.end {
                        $cp.data[($c as u32 - $cp.begin) as usize]
                    } else {
                        $c
                    }
                }};
            }

            match code_page {
                CodePage::USASCII => c,
                CodePage::UK => c, // technically incorrect
                CodePage::DECSpecialChars => code_page_lookup!(CODE_PAGE_0, c),
                CodePage::DOS437 => code_page_lookup!(CODE_PAGE_1, c),
            }
        } else {
            c
        };
        self.state.buffer.lines[self.state.cursor.y as usize][self.state.cursor.x as usize]
            .set(c, self.state.style);
        self.state.cursor.x += 1;
    }

    pub fn move_back(&mut self, count: u32) {
        for _ in 0..count {
            if (self.state.cursor.x as i32) - 1 < 0 {
                if self.state.cursor.y > 0 {
                    self.state.cursor.x = (self.width - 1) as i32;
                }
                self.state.cursor.y -= 1;
            } else {
                self.state.cursor.x -= 1;
            }
        }
        self.clamp_cursor();
    }

    pub fn delete_forward(&mut self, count: u32) {
        let count = count.min(self.width - (self.state.cursor.x as u32));
        let line = &mut self.state.buffer.lines[self.state.cursor.y as usize];
        for i in (self.state.cursor.x as u32)..self.width {
            let x = i + count;
            if x >= self.width {
                line[i as usize] = ScreenCell {
                    text: ' ',
                    style: self.state.style,
                };
            } else {
                line[i as usize] = line[x as usize];
            }
        }
    }

    pub fn erase_forward(&mut self, count: u32) {
        let end_index = self.width.min(self.state.cursor.x as u32 + count);
        let line = &mut self.state.buffer.lines[self.state.cursor.y as usize];
        for i in (self.state.cursor.x as u32)..end_index {
            line[i as usize] = ScreenCell {
                text: ' ',
                style: self.state.style,
            };
        }
    }

    pub fn insert_blanks(&mut self, count: u32) {
        let line = &mut self.state.buffer.lines[self.state.cursor.y as usize];
        let end_x = self.state.cursor.x + (count as i32) - 1;
        for i in (self.state.cursor.x..(self.width as i32)).rev() {
            let x = i - (count as i32);
            if x < 0 || x < end_x {
                line[i as usize] = ScreenCell {
                    text: ' ',
                    style: self.state.style,
                };
            } else {
                line[i as usize] = line[x as usize];
            }
        }
    }

    pub fn insert_lines(&mut self, count: u32) {
        let end_line = if (self.state.cursor.y as u32) + count >= self.state.scroll_margin_bottom {
            self.state.scroll_margin_bottom
        } else {
            (self.state.cursor.y as u32) + count
        };

        for y in (end_line..self.state.scroll_margin_bottom).rev() {
            self.state.buffer.lines[y as usize] =
                self.state.buffer.lines[(y - count) as usize].clone();
        }

        for y in (self.state.cursor.y as u32)..end_line {
            self.state.buffer.lines[y as usize] =
                ScreenBuffer::make_line(self.width as usize, self.state.style)
        }
    }

    pub fn delete_lines(&mut self, count: u32) {
        for y in (self.state.cursor.y as u32)..self.state.scroll_margin_bottom {
            if y + count >= self.state.scroll_margin_bottom {
                self.state.buffer.lines[y as usize] =
                    ScreenBuffer::make_line(self.width as usize, self.state.style);
            } else {
                self.state.buffer.lines[y as usize] =
                    self.state.buffer.lines[(y + count) as usize].clone();
            }
        }
    }

    fn handle_action(&mut self, action: Action) {
        use self::Action::*;

        match action {
            SetCursor(x, y) => {
                self.state.cursor.x = x as i32;
                self.state.cursor.y = y as i32;
                self.clamp_cursor();
            }
            SetCursorX(x) => {
                self.state.cursor.x = x as i32;
                self.clamp_cursor();
            }
            SetCursorLine(y) => {
                self.state.cursor.y = y as i32;
                self.clamp_cursor();
            }
            MoveCursor(x, y) => {
                self.state.cursor.x += x;
                self.state.cursor.y += y;
                self.clamp_cursor();
            }
            MoveCursorLine(y) => {
                self.state.cursor.x = 0;
                self.state.cursor.y += y;
                self.clamp_cursor();
            }
            MoveCursorLineWithScroll(y) => {
                self.state.cursor.y += y;
                if self.state.cursor.y < 0 {
                    let cursor_y = self.state.cursor.y;
                    self.scroll(cursor_y, true);
                } else if self.state.cursor.y >= self.height as i32 {
                    let cursor_y_diff = self.state.cursor.y - (self.height as i32);
                    self.scroll(cursor_y_diff + 1, true);
                }
            }
            ClearScreen(clear_type) => {
                let cursor_x = self.state.cursor.x as u32;
                let cursor_y = self.state.cursor.y as u32;
                let current_style = self.state.style.clone();

                match clear_type {
                    ClearType::All => self.clear_screen(),
                    ClearType::Before => {
                        self.clear_line_before(cursor_y, cursor_x, current_style.clone());
                        for y in 0..(self.state.cursor.y as u32) {
                            self.clear_line(y, current_style.clone());
                        }
                    }
                    ClearType::After => {
                        self.clear_line_after(cursor_y, cursor_x, current_style.clone());
                        for y in ((self.state.cursor.y + 1) as u32)..self.height {
                            self.clear_line(y, current_style.clone());
                        }
                    }
                }
            }
            ClearLine(clear_type) => {
                let cursor_x = self.state.cursor.x as u32;
                let cursor_y = self.state.cursor.y as u32;
                let current_style = self.state.style.clone();

                match clear_type {
                    ClearType::All => self.clear_line(cursor_y, current_style),
                    ClearType::Before => self.clear_line_before(cursor_y, cursor_x, current_style),
                    ClearType::After => self.clear_line_after(cursor_y, cursor_x, current_style),
                }
            }
            InsertLines(count) => self.insert_lines(count),
            DeleteLines(count) => self.delete_lines(count),
            DeleteForward(count) => self.delete_forward(count),
            EraseForward(count) => self.erase_forward(count),
            Scroll(count) => self.scroll(count, true),
            InsertBlanks(count) => self.insert_blanks(count),
            SetCursorStyle(style) => self.state.cursor.style = style,
            SaveCursor => self.state.saved_cursor = self.state.cursor,
            RestoreCursor => self.state.cursor = self.state.saved_cursor,
            SetCursorVisible(visible) => self.state.cursor.visible = visible,
            SetAltBuffer(enabled) => self.set_alt_buffer(enabled),
            SetScrollMargin(top, bottom) => {
                self.state.scroll_margin_top = top;
                self.state.scroll_margin_bottom = if bottom == 0 || bottom > self.height {
                    self.height
                } else {
                    bottom + 1
                }
            }
            ResetStyle => self.state.style.reset(),
            AddAttrs(attrs) => self.state.style.attrs |= attrs,
            RemoveAttrs(attrs) => self.state.style.attrs &= !attrs,
            SetColorFG(color) => {
                self.state.style.fg = color;
                self.state.style.attrs |= 1 << 0; // set attr_fg
            }
            SetColorBG(color) => {
                self.state.style.bg = color;
                self.state.style.attrs |= 1 << 1; // set attr_bg
            }
            ResetColorFG => self.state.style.attrs &= !(1 << 0),
            ResetColorBG => self.state.style.attrs &= !(1 << 1),
            SetWindowTitle(title) => self.state.title = title,
            SetRainbowMode(enabled) => self.state.rainbow = enabled,
            SetReverseVideo(enabled) => self.state.reverse_video = enabled,
            SetBracketedPaste(enabled) => self.state.bracketed_paste = enabled,
            SetMouseTracking(enabled) => self.state.track_mouse = enabled,
            SetLineSize(size) => self.state.buffer.line_sizes[self.state.cursor.y as usize] = size,
            SetCodePage(i, page) => self.state.charsets[i as usize] = page,
            SetCharSet(i) => self.state.charset = i,
            Bell => self.state.bell_id += 1,
            Backspace => self.move_back(1),
            NewLine => self.new_line(),
            Return => self.state.cursor.x = 0,
            Write(data) => {
                for character in data.chars() {
                    self.write_char(character);
                }
            }
            Resize(width, height) => {
                self.state.scroll_margin_bottom =
                    height.saturating_sub(self.height - self.state.scroll_margin_bottom);
                self.width = width;
                self.height = height;
                self.state
                    .buffer
                    .resize_lossy(width as usize, height as usize, self.state.style);
                self.state.alt_buffer.resize_lossy(
                    width as usize,
                    height as usize,
                    self.state.style,
                );
                self.clamp_cursor();
            }
            Interrupt => (),
            Tab => (),
            DeleteLine => (),
            DeleteWord => (),
        }
    }

    pub fn update_screen(&mut self) {
        for action in self.parser.drain_actions() {
            self.handle_action(action);
        }
        self.state.state_id += 1;
    }

    pub fn is_rainbow(&self) -> bool {
        self.state.rainbow
    }

    pub fn state_id(&self) -> u32 {
        self.state.state_id
    }

    pub fn title(&self) -> String {
        self.state.title.clone()
    }

    pub fn bell_id(&self) -> u32 {
        self.state.bell_id
    }

    pub fn write(&mut self, text: &str) {
        self.parser.write(text);
        self.update_screen();
    }

    pub fn cursor(&self) -> String {
        let cursor_x = if self.is_cursor_hanging() {
            self.state.cursor.x - 1
        } else {
            self.state.cursor.x
        };
        let mut cursor = String::new();
        cursor.push(encode_as_code_point(self.state.cursor.y as u32));
        cursor.push(encode_as_code_point(cursor_x as u32));
        if self.is_cursor_hanging() {
            cursor.push(encode_as_code_point(1));
        } else {
            cursor.push(encode_as_code_point(0));
        }
        cursor
    }

    pub fn attributes(&self) -> u32 {
        let mut attributes = 0u32;

        // show buttons/links
        attributes |= 1 << 7;
        attributes |= 1 << 8;

        if self.state.cursor.visible {
            attributes |= 1;
        }
        if self.state.track_mouse {
            attributes |= 1 << 5;
        }
        attributes |= (self.state.cursor.style as u32) << 9;

        if self.state.bracketed_paste {
            attributes |= 1 << 13;
        }
        if self.state.reverse_video {
            attributes |= 1 << 14;
        }

        attributes
    }

    pub fn current_code_page(&self) -> u32 {
        self.state.charset as u32
    }

    pub fn get_code_page(&self, i: usize) -> char {
        self.state.charsets[i].as_char()
    }

    pub fn is_tracking_mouse(&self) -> bool {
        self.state.track_mouse
    }

    pub fn scroll_margin(&self) -> [char; 2] {
        [
            encode_as_code_point(self.state.scroll_margin_top),
            encode_as_code_point(self.state.scroll_margin_bottom - 1),
        ]
    }

    pub fn reset_partial_screen(&mut self) {
        self.state.last_screen = Vec::new();
    }

    fn flatten_screen(&self) -> Vec<ScreenCell> {
        let mut screen_vec: Vec<ScreenCell> = Vec::new();

        for y in 0..(self.height as usize) {
            for x in 0..(self.width as usize) {
                screen_vec.push(self.state.buffer.lines[y][x].clone());
            }
        }

        screen_vec
    }

    fn screen_updates(&self, last: &[ScreenCell]) -> Vec<bool> {
        let mut updates: Vec<bool> = Vec::new();
        for _ in 0..(self.width * self.height) {
            updates.push(false);
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = (y * self.width + x) as usize;
                if let Some(last_cell) = last.get(cell) {
                    if &self.state.buffer.lines[y as usize][x as usize] != last_cell {
                        updates[cell] = true
                    }
                } else {
                    updates[cell] = true
                }
            }
        }

        updates
    }

    pub fn serialize_screen(&mut self, time: f64, full_update: bool) -> String {
        let mut data = String::from("S");

        // get frame
        let mut top = self.height as i32;
        let mut left = self.width as i32;
        let mut right = 0;
        let mut bottom = 0;

        let screen_updates = self.screen_updates(if full_update || self.state.rainbow {
            &[]
        } else {
            &self.state.last_screen
        });

        self.state.last_screen = self.flatten_screen();

        for cell in 0..(screen_updates.len() as i32) {
            if screen_updates[cell as usize] {
                let y = cell / (self.width as i32);
                let x = cell % (self.width as i32);
                if x < left {
                    left = x;
                }
                if x >= right {
                    right = x + 1;
                }
                if y < top {
                    top = y;
                }
                if y >= bottom {
                    bottom = y + 1;
                }
            }
        }

        if right <= left || bottom <= top {
            // return nothing
            return String::new();
        }

        // set frame to full size
        data.push(encode_as_code_point(top as u32));
        data.push(encode_as_code_point(left as u32));
        data.push(encode_as_code_point((bottom - top) as u32));
        data.push(encode_as_code_point((right - left) as u32));

        let mut last_style = CellStyle::new();

        for y in top..bottom {
            for x in left..right {
                let cell = &self.state.buffer.lines[y as usize][x as usize];
                let style = if self.state.rainbow {
                    CellStyle {
                        fg: get_rainbow_color(((x + y) as f64) / 10.0 + time),
                        bg: 0,
                        attrs: cell.style.attrs | 3,
                    }
                } else {
                    cell.style.clone()
                };

                if style != last_style {
                    let set_fg = style.fg != last_style.fg;
                    let set_bg = style.bg != last_style.bg;
                    let set_attrs = style.attrs != last_style.attrs;

                    if set_fg && set_bg {
                        if style.has_short_color() {
                            data.push('\x03');
                            data.push(encode_as_code_point((style.bg << 8) + style.fg));
                        } else {
                            data.push('\x05');
                            data.push_str(&encode_24color(style.fg));
                            data.push('\x06');
                            data.push_str(&encode_24color(style.bg));
                        }
                    } else if set_fg {
                        data.push('\x05');
                        data.push_str(&encode_24color(style.fg));
                    } else if set_bg {
                        data.push('\x06');
                        data.push_str(&encode_24color(style.bg));
                    }

                    if set_attrs {
                        data.push('\x04');
                        data.push(encode_as_code_point(style.attrs as u32));
                    }

                    last_style = style
                }
                data.push(cell.text);
            }
        }

        data
    }

    pub fn line_sizes(&self) -> String {
        let mut data = String::new();

        let mut index = 0;
        for size in &self.state.buffer.line_sizes {
            data.push(encode_as_code_point(
                (index << 3) | match size {
                    LineSize::Normal => 0,
                    LineSize::DoubleWidth => 0b001,
                    LineSize::DoubleHeightTop => 0b011,
                    LineSize::DoubleHeightBottom => 0b101,
                },
            ));
            index += 1;
        }

        let data_count = data.chars().count() as u32;
        data.insert(0, encode_as_code_point(data_count));
        data.insert(0, 'H');
        data
    }
}

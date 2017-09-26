pub mod sequence_parser;

use self::sequence_parser::{Action, SequenceParser, ClearType};
use std;

struct TerminalStyle {
    attrs: u16,
    fg: u8,
    bg: u8,
}

impl TerminalStyle {
    fn new() -> TerminalStyle {
        TerminalStyle {
            attrs: 0,
            fg: 0,
            bg: 0,
        }
    }
    fn reset(&mut self) {
        self.attrs = 0;
    }

    fn to_bytes(&self) -> u32 {
        ((self.attrs as u32) << 16) + ((self.bg as u32) << 8) + (self.fg as u32)
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
struct ScreenCell {
    text: char,
    style: u32,
}

struct ScreenBuffer {
    lines: Box<[Box<[ScreenCell]>]>,
}

impl ScreenBuffer {
    fn new(width: usize, height: usize) -> ScreenBuffer {
        let mut buf = ScreenBuffer { lines: Box::new([]) };
        buf.clear(width, height, 0);
        buf
    }

    fn make_line(width: usize, style: u32) -> Box<[ScreenCell]> {
        let mut line: Vec<ScreenCell> = Vec::new();
        for _ in 0..width {
            line.push(ScreenCell {
                text: ' ',
                style: style,
            })
        }
        line.into_boxed_slice()
    }

    fn clear(&mut self, width: usize, height: usize, style: u32) {
        let mut lines: Vec<Box<[ScreenCell]>> = Vec::new();

        for _ in 0..height {
            lines.push(ScreenBuffer::make_line(width, style))
        }

        self.lines = lines.into_boxed_slice();
    }

    fn clone_line(&self, ln: usize) -> Box<[ScreenCell]> {
        let original = &self.lines[ln];
        let mut clone: Vec<ScreenCell> = Vec::new();
        for x in 0..original.len() {
            clone.push(original[x].clone());
        }
        clone.into_boxed_slice()
    }
}

struct TerminalState {
    style: TerminalStyle,
    cursor: CursorState,
    saved_cursor: CursorState,
    track_mouse: bool,
    rainbow: bool,
    is_alt_buffer: bool,
    alt_buffer: ScreenBuffer,
    buffer: ScreenBuffer,
}

impl TerminalState {
    fn new() -> TerminalState {
        TerminalState {
            style: TerminalStyle::new(),
            cursor: CursorState::new(),
            saved_cursor: CursorState::new(),
            track_mouse: false,
            rainbow: false,
            is_alt_buffer: false,
            alt_buffer: ScreenBuffer::new(80, 25),
            buffer: ScreenBuffer::new(80, 25),
        }
    }
}

pub struct Terminal {
    pub width: u32,
    pub height: u32,
    parser: SequenceParser,
    state: TerminalState,
}

impl Terminal {
    pub fn new() -> Terminal {
        Terminal {
            width: 80,
            height: 25,
            parser: SequenceParser::new(),
            state: TerminalState::new(),
        }
    }
    pub fn is_cursor_hanging(&self) -> bool {
        self.state.cursor.x == self.width as i32
    }
    pub fn set_alt_buffer(&mut self, enabled: bool) {
        if enabled != self.state.is_alt_buffer {
            self.state.is_alt_buffer = enabled;

            std::mem::swap(&mut self.state.buffer, &mut self.state.alt_buffer);

            if enabled {
                self.clear_screen();
            }
        }
    }

    pub fn clear_screen(&mut self) {
        self.state.buffer.clear(self.width as usize,
                                self.height as usize,
                                self.state.style.to_bytes());
    }

    pub fn clear_line(&mut self, ln: u32, style: u32) {
        if ln >= self.height {
            return;
        }
        let line = &mut self.state.buffer.lines[ln as usize];
        for x in 0..self.width {
            line[x as usize].text = ' ';
            line[x as usize].style = style;
        }
    }

    pub fn clear_line_before(&mut self, ln: u32, col: u32, style: u32) {
        if ln >= self.height {
            return;
        }
        let line = &mut self.state.buffer.lines[ln as usize];
        for x in 0..(col + 1) {
            if x >= self.width {
                break;
            }
            line[x as usize].text = ' ';
            line[x as usize].style = style;
        }
    }

    pub fn clear_line_after(&mut self, ln: u32, col: u32, style: u32) {
        if ln >= self.height || col >= self.width {
            return;
        }
        let line = &mut self.state.buffer.lines[ln as usize];
        for x in col..self.width {
            line[x as usize].text = ' ';
            line[x as usize].style = style;
        }
    }

    pub fn scroll(&mut self, amount: i32, with_cursor: bool) {
        for y in 0..self.height {
            if (y as i32) + amount < 0 || ((y as i32) + amount) as u32 >= self.height {
                let line = ScreenBuffer::make_line(self.width as usize, 0);
                self.state.buffer.lines[y as usize] = line;
            } else {
                self.state.buffer.lines[y as usize] =
                    self.state.buffer.clone_line(((y as i32) + amount) as usize);
            }
        }
        if with_cursor {
            self.state.cursor.y += amount;
            self.clamp_cursor();
        }
    }

    pub fn clamp_cursor(&mut self) {
        if self.state.cursor.x < 0 {
            self.state.cursor.x = 0;
        }
        if self.state.cursor.x > self.width as i32 {
            self.state.cursor.x = self.width as i32;
        }
        if self.state.cursor.y < 0 {
            self.state.cursor.y = 0;
        }
        if self.state.cursor.y >= self.height as i32 {
            self.state.cursor.y = (self.height - 1) as i32;
        }
    }

    pub fn new_line(&mut self) {
        self.state.cursor.y += 1;
        if self.state.cursor.y >= self.height as i32 {
            self.scroll(1, true);
        }
    }

    pub fn write_char(&mut self, character: char) {
        if self.state.cursor.x >= self.width as i32 {
            self.state.cursor.x = 0;
            self.new_line();
        }
        let mut cell = &mut self.state.buffer.lines[self.state.cursor.y as usize][self.state.cursor.x as
                            usize];
        cell.text = character;
        cell.style = self.state.style.to_bytes();
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
    }

    pub fn delete_forward(&mut self, count: u32) {
        let count = std::cmp::min(count, self.width - (self.state.cursor.x as u32));
        let mut line = &mut self.state.buffer.lines[self.state.cursor.y as usize];
        for i in 0..count {
            let x = self.state.cursor.x + i as i32;
            if x + 1 <= self.width as i32 {
                line[x as usize] = ScreenCell {
                    text: ' ',
                    style: 0,
                };
            } else {
                line[x as usize] = line[(x as usize) + 1].clone();
            }
        }
    }

    pub fn insert_blanks(&mut self, count: u32) {
        let mut line = &mut self.state.buffer.lines[self.state.cursor.y as usize];
        for i in 0..count {
            let x = self.state.cursor.x + i as i32;
            line[x as usize] = ScreenCell {
                text: ' ',
                style: self.state.style.to_bytes(),
            };
        }
    }

    pub fn insert_lines(&mut self, count: u32) {
        let end_line = if (self.state.cursor.y as u32) + count >= self.height {
            self.height
        } else {
            (self.state.cursor.y as u32) + count
        };

        for y in (end_line..self.height).rev() {
            self.state.buffer.lines[y as usize] = self.state.buffer.lines[(y - count) as usize]
                .clone();
        }

        for y in (self.state.cursor.y as u32)..((self.state.cursor.y as u32) + count) {
            self.state.buffer.lines[y as usize] =
                ScreenBuffer::make_line(self.width as usize, self.state.style.to_bytes())
        }
    }

    pub fn delete_lines(&mut self, count: u32) {
        for y in (self.state.cursor.y as u32)..self.height {
            if y + count >= self.height {
                self.state.buffer.lines[y as usize] = ScreenBuffer::make_line(self.width as usize,
                                                                              0);
            } else {
                self.state.buffer.lines[y as usize] = self.state.buffer.lines[(y + count) as usize]
                    .clone();
            }
        }
    }

    fn handle_action(&mut self, action: Action) {
        use terminal::sequence_parser::Action::*;

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
            ClearScreen(clear_type) => {
                let cursor_x = self.state.cursor.x as u32;
                let cursor_y = self.state.cursor.y as u32;
                let current_style = self.state.style.to_bytes();

                if clear_type == ClearType::All {
                    self.clear_screen();
                } else if clear_type == ClearType::Before {
                    self.clear_line_before(cursor_y, cursor_x, current_style);
                    for y in 0..(self.state.cursor.y as u32) {
                        self.clear_line(y, current_style);
                    }
                } else if clear_type == ClearType::After {
                    self.clear_line_after(cursor_y, cursor_x, current_style);
                    for y in ((self.state.cursor.y + 1) as u32)..self.height {
                        self.clear_line(y, current_style);
                    }
                }
            }
            ClearLine(clear_type) => {
                let cursor_x = self.state.cursor.x as u32;
                let cursor_y = self.state.cursor.y as u32;
                let current_style = self.state.style.to_bytes();

                if clear_type == ClearType::All {
                    self.clear_line(cursor_y, current_style);
                } else if clear_type == ClearType::Before {
                    self.clear_line_before(cursor_y, cursor_x, current_style);
                } else if clear_type == ClearType::After {
                    self.clear_line_after(cursor_y, cursor_x, current_style);
                }
            }
            InsertLines(count) => self.insert_lines(count),
            DeleteLines(count) => self.delete_lines(count),
            DeleteForward(count) => self.delete_forward(count),
            Scroll(count) => self.scroll(count, true),
            InsertBlanks(count) => self.insert_blanks(count),
            SetCursorStyle(style) => self.state.cursor.style = style,
            SaveCursor => self.state.saved_cursor = self.state.cursor.clone(),
            RestoreCursor => {
                self.state.cursor = self.state.saved_cursor.clone();
            }
            SetCursorVisible(visible) => self.state.cursor.visible = visible,
            SetAltBuffer(enabled) => self.set_alt_buffer(enabled),
            ResetStyle => self.state.style.reset(),
            AddAttrs(attrs) => self.state.style.attrs |= attrs,
            RemoveAttrs(attrs) => self.state.style.attrs &= !attrs,
            SetColorFG(color) => {
                self.state.style.fg = color as u8;
                self.state.style.attrs |= 1 << 8; // set has_fg
            }
            SetColorBG(color) => {
                self.state.style.bg = color as u8;
                self.state.style.attrs |= 1 << 9; // set has_bg
            }
            ResetColorFG => self.state.style.attrs &= !(1 << 8),
            ResetColorBG => self.state.style.attrs &= !(1 << 9),
            SetWindowTitle(title) => {
                // TODO
                println!("Set title: {}", title);
            }
            SetRainbowMode(enabled) => self.state.rainbow = enabled,
            Bell => {
                // TODO
                println!("bell");
            }
            Backspace => self.move_back(1),
            NewLine => self.new_line(),
            Return => self.state.cursor.x = 0,
            Write(data) => {
                for character in data.chars() {
                    self.write_char(character);
                }
            }
            _ => (),
        }
    }

    pub fn update_screen(&mut self) {
        for action in self.parser.pop_stack() {
            self.handle_action(action);
        }
        self.parser.stack = Vec::new();
    }

    pub fn write(&mut self, text: String) {
        self.parser.write(&text);
        self.update_screen();
    }

    pub fn serialize(&self) -> String {
        let mut data = String::from("S");
        data.push(std::char::from_u32(self.height + 1).unwrap());
        data.push(std::char::from_u32(self.width + 1).unwrap());
        data.push(std::char::from_u32((self.state.cursor.y as u32) + 1).unwrap());
        data.push(std::char::from_u32((self.state.cursor.x as u32) + 1).unwrap());

        let mut attributes = if self.state.cursor.visible {
            1u32
        } else {
            0u32
        };
        if self.state.track_mouse {
            attributes |= 3 << 5;
        }
        attributes |= 3 << 7;
        attributes |= (self.state.cursor.style as u32) << 9;
        data.push(std::char::from_u32(attributes + 1).unwrap());

        let mut last_style = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &self.state.buffer.lines[y as usize][x as usize];
                let style = cell.style;
                if style != last_style {
                    let fg = style & 0xFF;
                    let bg = (style >> 8) & 0xFF;
                    let attrs = (style >> 16) & 0xFFFF;
                    let set_fg = fg != (last_style & 0xFF);
                    let set_bg = bg != ((last_style >> 8) & 0xFF);
                    let set_attrs = attrs != ((last_style >> 16) & 0xFFFF);

                    if set_fg && set_bg {
                        data.push('\x03');
                        data.push(std::char::from_u32((style & 0xFFFF) + 1).unwrap());
                    } else if set_fg {
                        data.push('\x05');
                        data.push(std::char::from_u32(fg + 1).unwrap());
                    } else if set_bg {
                        data.push('\x06');
                        data.push(std::char::from_u32(bg + 1).unwrap());
                    }

                    if set_attrs {
                        data.push('\x04');
                        data.push(std::char::from_u32(attrs + 1).unwrap());
                    }

                    last_style = style
                }
                data.push(cell.text);
            }
        }

        data
    }
}

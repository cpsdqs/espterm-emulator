use std::mem;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum SequenceType {
    None,
    ESC,
    ANSI,
    OSC,
    OneChar,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClearType {
    Before,
    After,
    All,
}

impl From<i32> for ClearType {
    fn from(num: i32) -> ClearType {
        match num {
            1 => ClearType::Before,
            2 => ClearType::All,
            _ => ClearType::After,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineSize {
    Normal,
    DoubleWidth,
    DoubleHeightTop,
    DoubleHeightBottom,
}

impl Default for LineSize {
    fn default() -> LineSize {
        LineSize::Normal
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodePage {
    DECSpecialChars,
    DOS437,
    UK,
    USASCII,
}

impl CodePage {
    pub fn as_char(self) -> char {
        match self {
            CodePage::DECSpecialChars => '0',
            CodePage::DOS437 => '1',
            CodePage::UK => 'A',
            CodePage::USASCII => 'B',
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    SetCursor(u32, u32),
    SetCursorX(u32),
    SetCursorLine(u32),
    MoveCursor(i32, i32),
    MoveCursorLine(i32),
    MoveCursorLineWithScroll(i32),
    ClearScreen(ClearType),
    ClearLine(ClearType),
    InsertLines(u32),
    DeleteLines(u32),
    DeleteForward(u32),
    EraseForward(u32),
    Scroll(i32),
    InsertBlanks(u32),
    SetCursorStyle(u8),
    SaveCursor,
    RestoreCursor,
    SetCursorVisible(bool),
    SetAltBuffer(bool),
    SetScrollMargin(u32, u32),
    ResetStyle,
    AddAttrs(u16),
    RemoveAttrs(u16),
    SetColorFG(u32),
    SetColorBG(u32),
    ResetColorFG,
    ResetColorBG,
    SetReverseVideo(bool),
    SetBracketedPaste(bool),
    SetWindowTitle(String),
    SetRainbowMode(bool),
    SetMouseTracking(bool),
    SetLineSize(LineSize),
    SetCodePage(u8, CodePage),
    SetCharSet(u8),
    Interrupt,
    Bell,
    Backspace,
    Tab,
    NewLine,
    Return,
    DeleteLine,
    DeleteWord,
    Write(String),
    Resize(u32, u32),
}

/// Escape sequence parser.
pub struct SeqParser {
    /// Current sequence type.
    seq_type: SequenceType,

    /// Current sequence.
    sequence: String,

    /// Accumulated actions.
    actions: Vec<Action>,
}

/// Convenience for Vec<Option<T>>
trait OptionalVec<T> {
    fn get_opt(&self, index: usize) -> Option<T>;
}
impl OptionalVec<i32> for Vec<Option<i32>> {
    fn get_opt(&self, index: usize) -> Option<i32> {
        *self.get(index).unwrap_or(&None)
    }
}

impl SeqParser {
    pub fn new() -> SeqParser {
        SeqParser {
            seq_type: SequenceType::None,
            sequence: String::new(),
            actions: Vec::new(),
        }
    }

    pub fn drain_actions(&mut self) -> Vec<Action> {
        mem::replace(&mut self.actions, Vec::new())
    }

    fn reset_state(&mut self) {
        self.seq_type = SequenceType::None;
        self.sequence = String::new();
    }

    pub fn apply_seq(&mut self) {
        let seq = mem::replace(&mut self.sequence, String::new());
        self.reset_state();
        let first_char = seq.chars().next().unwrap_or(0 as char);

        match first_char {
            '[' => {
                // ANSI
                let chars: Vec<_> = seq.chars().collect();

                // ANSI action is the last character (like m in \e[0m)
                let action = *chars.last().unwrap();

                // content between [ and action
                let content = if chars.len() > 2 {
                    (chars[1..chars.len() - 1]).iter().collect()
                } else {
                    String::new()
                };

                // often, ANSI uses numbers separated by semicolons so here's a vec for that
                let numbers: Vec<Option<i32>> = if content.trim().len() == 0 {
                    Vec::new()
                } else {
                    content
                        .trim()
                        .split(';')
                        .map(|x| match x.parse::<i32>() {
                            Ok(x) => Some(x),
                            Err(_) => None,
                        })
                        .collect()
                };

                match action {
                    'H' | 'f' => {
                        let y = numbers.get_opt(0).map(|x| x - 1).unwrap_or(0);
                        let x = numbers.get_opt(1).map(|x| x - 1).unwrap_or(0);
                        self.actions.push(Action::SetCursor(x as u32, y as u32));
                    }
                    'A' => self
                        .actions
                        .push(Action::MoveCursor(0, -numbers.get_opt(0).unwrap_or(1))),
                    'B' => self
                        .actions
                        .push(Action::MoveCursor(0, numbers.get_opt(0).unwrap_or(1))),
                    'C' => self
                        .actions
                        .push(Action::MoveCursor(numbers.get_opt(0).unwrap_or(1), 0)),
                    'D' => self
                        .actions
                        .push(Action::MoveCursor(-numbers.get_opt(0).unwrap_or(1), 0)),
                    'E' => self
                        .actions
                        .push(Action::MoveCursorLine(numbers.get_opt(0).unwrap_or(1))),
                    'F' => self
                        .actions
                        .push(Action::MoveCursorLine(-numbers.get_opt(0).unwrap_or(1))),
                    'G' => self.actions.push(Action::SetCursorX(
                        (numbers.get_opt(0).unwrap_or(1) as u32) - 1,
                    )),
                    'J' => {
                        let clear_type: ClearType = numbers.get_opt(0).unwrap_or(0).into();
                        self.actions.push(Action::ClearScreen(clear_type));
                    }
                    'K' => {
                        let clear_type: ClearType = numbers.get_opt(0).unwrap_or(0).into();
                        self.actions.push(Action::ClearLine(clear_type));
                    }
                    'L' => self
                        .actions
                        .push(Action::InsertLines(numbers.get_opt(0).unwrap_or(1) as u32)),
                    'M' => self
                        .actions
                        .push(Action::DeleteLines(numbers.get_opt(0).unwrap_or(1) as u32)),
                    'P' => self
                        .actions
                        .push(Action::DeleteForward(numbers.get_opt(0).unwrap_or(1) as u32)),
                    'S' => self.actions.push(Action::Scroll(1)),
                    'T' => self.actions.push(Action::Scroll(-1)),
                    'X' => self
                        .actions
                        .push(Action::EraseForward(numbers.get_opt(0).unwrap_or(1) as u32)),
                    '@' => self
                        .actions
                        .push(Action::InsertBlanks(numbers.get_opt(0).unwrap_or(1) as u32)),
                    'd' => self
                        .actions
                        .push(Action::SetCursorLine(numbers.get_opt(0).unwrap_or(1) as u32)),
                    'q' => self
                        .actions
                        .push(Action::SetCursorStyle(numbers.get_opt(0).unwrap_or(1) as u8)),
                    'r' => {
                        let top = numbers.get_opt(0).map(|x| x - 1).unwrap_or(0);
                        let bottom = numbers.get_opt(1).map(|x| x - 1).unwrap_or(0);
                        self.actions
                            .push(Action::SetScrollMargin(top as u32, bottom as u32));
                    }
                    's' => self.actions.push(Action::SaveCursor),
                    'u' => self.actions.push(Action::RestoreCursor),
                    'm' => {
                        if numbers.len() == 0 {
                            self.actions.push(Action::ResetStyle);
                        } else {
                            let mut numbers = numbers.into_iter();
                            loop {
                                if let Some(sgr_type) = numbers.next() {
                                    let sgr_type = sgr_type.unwrap_or(0);
                                    match sgr_type {
                                        // reset
                                        0 => self.actions.push(Action::ResetStyle),
                                        // bold
                                        1 => self.actions.push(Action::AddAttrs(1 << 2)),
                                        // faint
                                        2 => self.actions.push(Action::AddAttrs(1 << 9)),
                                        // italic
                                        3 => self.actions.push(Action::AddAttrs(1 << 6)),
                                        // underline
                                        4 => self.actions.push(Action::AddAttrs(1 << 3)),
                                        // blink
                                        5 | 6 => self.actions.push(Action::AddAttrs(1 << 5)),
                                        // invert
                                        7 => self.actions.push(Action::AddAttrs(1 << 4)),
                                        // strike
                                        9 => self.actions.push(Action::AddAttrs(1 << 7)),
                                        // fraktur
                                        20 => self.actions.push(Action::AddAttrs(1 << 10)),
                                        // remove bold
                                        21 => self.actions.push(Action::RemoveAttrs(1 << 2)),
                                        // remove bold and faint
                                        22 => self
                                            .actions
                                            .push(Action::RemoveAttrs((1 << 2) | (1 << 9))),
                                        // remove italic and fraktur
                                        23 => self
                                            .actions
                                            .push(Action::RemoveAttrs((1 << 6) | (1 << 10))),
                                        // remove underline
                                        24 => self.actions.push(Action::RemoveAttrs(1 << 3)),
                                        // remove blink
                                        25 => self.actions.push(Action::RemoveAttrs(1 << 5)),
                                        // remove inverse
                                        27 => self.actions.push(Action::RemoveAttrs(1 << 4)),
                                        // set foreground
                                        color @ 30...37 => {
                                            self.actions.push(Action::SetColorFG(color as u32 % 10))
                                        }
                                        // set background
                                        color @ 40...47 => {
                                            self.actions.push(Action::SetColorBG(color as u32 % 10))
                                        }
                                        // reset foreground
                                        39 => self.actions.push(Action::ResetColorFG),
                                        // reset background
                                        49 => self.actions.push(Action::ResetColorBG),
                                        // set bright foreground
                                        color @ 90...97 => self
                                            .actions
                                            .push(Action::SetColorFG((color as u32 % 10) + 8)),
                                        // set bright background
                                        color @ 100...107 => self
                                            .actions
                                            .push(Action::SetColorBG((color as u32 % 10) + 8)),
                                        38 | 48 => {
                                            let is_fg = sgr_type == 38;
                                            if let Some(type_mod) = numbers.next() {
                                                let type_mod = type_mod.unwrap_or(0);
                                                match type_mod {
                                                    2 => {
                                                        if let Some(r) = numbers.next() {
                                                            if let Some(g) = numbers.next() {
                                                                if let Some(b) = numbers.next() {
                                                                    let r = r.unwrap_or(0);
                                                                    let g = g.unwrap_or(0);
                                                                    let b = b.unwrap_or(0);
                                                                    let color: u32 = ((r << 16) + (g << 8) + b + 256) as u32;
                                                                    if is_fg {
                                                                        self.actions.push(
                                                                            Action::SetColorFG(
                                                                                color,
                                                                            ),
                                                                        );
                                                                    } else {
                                                                        self.actions.push(
                                                                            Action::SetColorBG(
                                                                                color,
                                                                            ),
                                                                        );
                                                                    }
                                                                } else {
                                                                    break;
                                                                }
                                                            } else {
                                                                break;
                                                            }
                                                        } else {
                                                            break;
                                                        }
                                                    }
                                                    5 => {
                                                        if let Some(color) = numbers.next() {
                                                            let color = color.unwrap_or(0);
                                                            if is_fg {
                                                                self.actions.push(
                                                                    Action::SetColorFG(
                                                                        color as u32,
                                                                    ),
                                                                );
                                                            } else {
                                                                self.actions.push(
                                                                    Action::SetColorBG(
                                                                        color as u32,
                                                                    ),
                                                                );
                                                            }
                                                        } else {
                                                            break;
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                            } else {
                                                break;
                                            }
                                        }
                                        _ => {
                                            println!("Unhandled SGR: {}", seq);
                                        }
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    'h' | 'l' => {
                        match &*content {
                            "?5" => self.actions.push(Action::SetReverseVideo(action == 'h')),
                            "?25" => self.actions.push(Action::SetCursorVisible(action == 'h')),
                            "?1000" => self.actions.push(Action::SetMouseTracking(action == 'h')),
                            "?1049" => {
                                // TODO: proper behavior
                                self.actions.push(Action::SetAltBuffer(action == 'h'));
                            }
                            "?2004" => {
                                self.actions.push(Action::SetBracketedPaste(action == 'h'));
                            }
                            // TODO
                            _ => {
                                println!("Unhandled mode: {}", seq);
                            }
                        };
                    }
                    't' => match numbers.get_opt(0).unwrap_or(0) {
                        8 => {
                            let height = numbers.get_opt(1).unwrap_or(24).max(1).min(65535);
                            let width = numbers.get_opt(2).unwrap_or(80).max(10).min(65535);
                            self.actions
                                .push(Action::Resize(width as u32, height as u32));
                        }
                        _ => println!("Unhandled ANSI sequence: {}", seq),
                    },
                    _ => {
                        println!("Unhandled ANSI sequence: {}", seq);
                    }
                }
            }
            ']' => {
                // OSC
                let mut data = seq[1..].split(';');
                if let Some(osc_type) = data.next() {
                    match osc_type {
                        "0" => {
                            // window title
                            if let Some(title) = data.next() {
                                self.actions
                                    .push(Action::SetWindowTitle(String::from(title)));
                            }
                        }
                        "360" => {
                            // rainbow mode
                            let mut enabled = false;
                            if let Some(arg) = data.next() {
                                if arg == "1" {
                                    enabled = true;
                                }
                            }
                            self.actions.push(Action::SetRainbowMode(enabled));
                        }
                        _ => (),
                    }
                }
            }
            '(' | ')' => {
                let mut chars = seq.chars();
                let g: u8 = match chars.next().unwrap_or('(') {
                    '(' => 0,
                    ')' => 1,
                    _ => 0,
                };

                match chars.next() {
                    Some('0') => self
                        .actions
                        .push(Action::SetCodePage(g, CodePage::DECSpecialChars)),
                    Some('1') => self.actions.push(Action::SetCodePage(g, CodePage::DOS437)),
                    Some('A') => self.actions.push(Action::SetCodePage(g, CodePage::UK)),
                    Some('B') => self.actions.push(Action::SetCodePage(g, CodePage::USASCII)),
                    _ => println!("Unhandled code page: {}", seq),
                }
            }
            'D' => self.actions.push(Action::MoveCursorLineWithScroll(1)),
            'M' => self.actions.push(Action::MoveCursorLineWithScroll(-1)),
            '\u{f}' => self.actions.push(Action::SetCharSet(0)),
            '\u{e}' => self.actions.push(Action::SetCharSet(1)),
            '#' => {
                let mut chars = seq.chars();
                chars.next(); // skip #
                match chars.next() {
                    Some('3') => self
                        .actions
                        .push(Action::SetLineSize(LineSize::DoubleHeightTop)),
                    Some('4') => self
                        .actions
                        .push(Action::SetLineSize(LineSize::DoubleHeightBottom)),
                    Some('5') => self.actions.push(Action::SetLineSize(LineSize::Normal)),
                    Some('6') => self
                        .actions
                        .push(Action::SetLineSize(LineSize::DoubleWidth)),
                    _ => println!("Unhandled #: {}", seq),
                }
            }
            'c' => {
                self.drain_actions();
                self.reset_state();
                self.actions.push(Action::SetCursor(0, 0));
                self.actions.push(Action::ResetStyle);
                self.actions.push(Action::ClearScreen(ClearType::All));
            }
            _ => {
                println!("Unhandled escape: {}", seq);
            }
        };
    }

    pub fn write(&mut self, data: &str) {
        for c in data.chars() {
            let code_point = c as u32;

            if code_point == 0x1b && self.seq_type == SequenceType::None {
                self.seq_type = SequenceType::ESC;
            } else if code_point == 0x9d && self.seq_type == SequenceType::None {
                self.seq_type = SequenceType::OSC;
                self.sequence = String::from("]");
            } else if c == '[' && self.seq_type == SequenceType::ESC {
                self.seq_type = SequenceType::ANSI;
                self.sequence = String::from("[");
            } else if c == ']' && self.seq_type == SequenceType::ESC {
                self.seq_type = SequenceType::OSC;
                self.sequence = String::from("]");
            } else if (c == '(' || c == ')' || c == '#') && self.seq_type == SequenceType::ESC {
                self.seq_type = SequenceType::OneChar;
                self.sequence.push(c);
            } else if self.seq_type != SequenceType::None
                && self.seq_type != SequenceType::ESC
                && (code_point == 0x1b || code_point == 0x9c || code_point == 0x07)
            {
                self.apply_seq();
                if code_point == 0x1b {
                    self.seq_type = SequenceType::ESC;
                }
            } else if self.seq_type == SequenceType::ANSI && '\x40' <= c && c <= '\x7e' {
                self.sequence.push(c);
                self.apply_seq();
            } else if self.seq_type == SequenceType::ESC {
                if c == '\\' {
                    // ST
                    self.reset_state()
                } else {
                    self.sequence.push(c);
                    self.apply_seq();
                }
            } else if self.seq_type != SequenceType::None {
                self.sequence.push(c);
                if self.seq_type == SequenceType::OneChar {
                    self.apply_seq();
                }
            } else {
                match code_point {
                    0...2 => (),
                    3 => self.actions.push(Action::Interrupt),
                    4...6 => (),
                    7 => self.actions.push(Action::Bell),
                    8 => self.actions.push(Action::Backspace),
                    9 => self.actions.push(Action::Tab),
                    0xA => self.actions.push(Action::NewLine),
                    0xD => self.actions.push(Action::Return),
                    0x15 => self.actions.push(Action::DeleteLine),
                    0x17 => self.actions.push(Action::DeleteWord),
                    _ => {
                        self.actions.push(Action::Write(c.to_string()));
                    }
                }
            }
        }
    }
}

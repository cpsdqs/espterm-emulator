#[derive(PartialEq)]
enum SequenceType {
    None,
    ESC,
    ANSI,
    OSC,
    OneChar,
}

struct State {
    sequence_type: SequenceType,
    sequence: String,
}

impl State {
    fn new() -> State {
        State {
            sequence_type: SequenceType::None,
            sequence: String::new(),
        }
    }
    fn reset(&mut self) {
        self.sequence_type = SequenceType::None;
        self.sequence = String::new();
    }
}

#[derive(PartialEq)]
pub enum ClearType {
    Before,
    After,
    All,
}

pub enum Action {
    SetCursor(u32, u32),
    SetCursorX(u32),
    MoveCursor(i32, i32),
    MoveCursorLine(i32),
    ClearScreen(ClearType),
    ClearLine(ClearType),
    InsertLines(u32),
    DeleteLines(u32),
    DeleteForward(u32),
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
    SetWindowTitle(String),
    SetRainbowMode(bool),
    Interrupt,
    Bell,
    Backspace,
    Tab,
    NewLine,
    Return,
    DeleteLine,
    DeleteWord,
    Write(String),
}

pub struct SequenceParser {
    state: State,
    pub stack: Vec<Action>,
}

impl SequenceParser {
    pub fn new() -> SequenceParser {
        SequenceParser {
            state: State::new(),
            stack: vec![],
        }
    }

    pub fn pop_stack(&mut self) -> Vec<Action> {
        let mut stack: Vec<Action> = Vec::new();

        self.stack.reverse();
        loop {
            if let Some(action) = self.stack.pop() {
                stack.push(action);
            } else {
                break;
            }
        }

        stack
    }

    pub fn apply_sequence(&mut self) {
        {
            let sequence = &self.state.sequence;
            let bytes = sequence.as_bytes();
            let first_char = if bytes.len() == 0 {
                0
            } else {
                bytes[0]
            };

            match first_char {
                b'[' => {
                    // ANSI
                    let action = bytes[bytes.len() - 1];
                    let content = if bytes.len() > 1 {
                        // weird (it works, shrug)
                        let a = String::from_utf8_lossy(&bytes[1..bytes.len() - 1]);
                        String::from(a)
                    } else {
                        String::new()
                    };

                    let numbers: Vec<i32> = if content.trim().len() == 0 {
                        Vec::new()
                    } else {
                        content.trim().split(';')
                            .map(|x| match x.parse::<i32>() {
                                Ok(x) => x,
                                Err(_) => 0,
                            }).collect()
                    };

                    let num_or_zero = match numbers.get(0) {
                        Some(x) => *x,
                        None => 0,
                    };
                    let num_or_one = match numbers.get(0) {
                        Some(x) => *x,
                        None => 1,
                    };

                    match action {
                        b'H' | b'f' => {
                            let y = match numbers.get(0) {
                                Some(x) => *x - 1,
                                None => 0,
                            };
                            let x = match numbers.get(1) {
                                Some(x) => *x - 1,
                                None => 0,
                            };
                            self.stack.push(Action::SetCursor(x as u32, y as u32));
                        }
                        b'A' => self.stack.push(Action::MoveCursor(0, -num_or_one)),
                        b'B' => self.stack.push(Action::MoveCursor(0, num_or_one)),
                        b'C' => self.stack.push(Action::MoveCursor(num_or_one, 0)),
                        b'D' => self.stack.push(Action::MoveCursor(-num_or_one, 0)),
                        b'E' => self.stack.push(Action::MoveCursorLine(num_or_one)),
                        b'F' => self.stack.push(Action::MoveCursorLine(-num_or_one)),
                        b'G' => self.stack.push(Action::SetCursorX((num_or_one as u32) - 1)),
                        b'J' => {
                            let clear_type = match num_or_zero {
                                1 => ClearType::Before,
                                2 => ClearType::All,
                                _ => ClearType::After,
                            };
                            self.stack.push(Action::ClearScreen(clear_type));
                        }
                        b'K' => {
                            let clear_type = match num_or_zero {
                                1 => ClearType::Before,
                                2 => ClearType::All,
                                _ => ClearType::After,
                            };
                            self.stack.push(Action::ClearLine(clear_type));
                        }
                        b'L' => self.stack.push(Action::InsertLines(num_or_one as u32)),
                        b'M' => self.stack.push(Action::DeleteLines(num_or_one as u32)),
                        b'P' => self.stack.push(Action::DeleteForward(num_or_one as u32)),
                        b'S' => self.stack.push(Action::Scroll(1)),
                        b'T' => self.stack.push(Action::Scroll(-1)),
                        b'@' => self.stack.push(Action::InsertBlanks(num_or_one as u32)),
                        b'q' => self.stack.push(Action::SetCursorStyle(num_or_one as u8)),
                        b'r' => {
                            let top = match numbers.get(0) {
                                Some(x) => *x - 1,
                                None => 0,
                            };
                            let bottom = match numbers.get(1) {
                                Some(x) => *x - 1,
                                None => 0,
                            };
                            self.stack.push(Action::SetScrollMargin(top as u32, bottom as u32));
                        },
                        b's' => self.stack.push(Action::SaveCursor),
                        b'u' => self.stack.push(Action::RestoreCursor),
                        b'm' => {
                            if numbers.len() == 0 {
                                self.stack.push(Action::ResetStyle);
                            } else {
                                let mut numbers = numbers.into_iter();
                                loop {
                                    if let Some(sgr_type) = numbers.next() {
                                        match sgr_type {
                                            // reset
                                            0 => self.stack.push(Action::ResetStyle),
                                            // bold
                                            1 => self.stack.push(Action::AddAttrs(1 << 2)),
                                            // faint
                                            2 => self.stack.push(Action::AddAttrs(1 << 9)),
                                            // italic
                                            3 => self.stack.push(Action::AddAttrs(1 << 6)),
                                            // underline
                                            4 => self.stack.push(Action::AddAttrs(1 << 3)),
                                            // blink
                                            5 | 6 => self.stack.push(Action::AddAttrs(1 << 5)),
                                            // invert
                                            7 => self.stack.push(Action::AddAttrs(1 << 4)),
                                            // strike
                                            9 => self.stack.push(Action::AddAttrs(1 << 7)),
                                            // fraktur
                                            20 => self.stack.push(Action::AddAttrs(1 << 10)),
                                            // remove bold
                                            21 => self.stack.push(Action::RemoveAttrs(1 << 2)),
                                            // remove bold and faint
                                            22 => self.stack.push(Action::RemoveAttrs((1 << 2) | (1 << 9))),
                                            // remove italic and fraktur
                                            23 => self.stack.push(Action::RemoveAttrs((1 << 6) | (1 << 10))),
                                            // remove underline
                                            24 => self.stack.push(Action::RemoveAttrs(1 << 3)),
                                            // remove blink
                                            25 => self.stack.push(Action::RemoveAttrs(1 << 5)),
                                            // remove inverse
                                            27 => self.stack.push(Action::RemoveAttrs(1 << 4)),
                                            // set foreground
                                            color @ 30...37 => {
                                                self.stack.push(Action::SetColorFG(color as u32 % 10))
                                            }
                                            // set background
                                            color @ 40...47 => {
                                                self.stack.push(Action::SetColorBG(color as u32 % 10))
                                            }
                                            // reset foreground
                                            39 => self.stack.push(Action::ResetColorFG),
                                            // reset background
                                            49 => self.stack.push(Action::ResetColorBG),
                                            // set bright foreground
                                            color @ 90...97 => {
                                                self.stack.push(Action::SetColorFG((color as u32 % 10) + 8))
                                            }
                                            // set bright background
                                            color @ 100...107 => {
                                                self.stack.push(Action::SetColorBG((color as u32 % 10) + 8))
                                            }
                                            38 | 48 => {
                                                let is_fg = sgr_type == 38;
                                                if let Some(type_mod) = numbers.next() {
                                                    match type_mod {
                                                        2 => {
                                                            if let Some(r) = numbers.next() {
                                                                if let Some(g) = numbers.next() {
                                                                    if let Some(b) = numbers.next() {
                                                                        let color: u32 = ((r << 16) + (g << 8) + b + 256) as u32;
                                                                        if is_fg {
                                                                            self.stack.push(Action::SetColorFG(color));
                                                                        } else {
                                                                            self.stack.push(Action::SetColorBG(color));
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
                                                                if is_fg {
                                                                    self.stack.push(Action::SetColorFG(color as u32));
                                                                } else {
                                                                    self.stack.push(Action::SetColorBG(color as u32));
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
                                            _ => (),
                                        }
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                        b'h' | b'l' => {
                            match &content as &str {
                                "?25h" => self.stack.push(Action::SetCursorVisible(true)),
                                "?25l" => self.stack.push(Action::SetCursorVisible(false)),
                                // TODO: proper behavior
                                "?1049h" => self.stack.push(Action::SetAltBuffer(true)),
                                "?1049l" => self.stack.push(Action::SetAltBuffer(false)),
                                // TODO
                                _ => ()
                            };
                        }
                        _ => (),
                    }
                }
                b']' => {
                    // OSC
                    let mut data = sequence[1..].split(';');
                    if let Some(osc_type) = data.next() {
                        match osc_type {
                            "0" => {
                                // window title
                                if let Some(title) = data.next() {
                                    self.stack.push(Action::SetWindowTitle(String::from(title)));
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
                                self.stack.push(Action::SetRainbowMode(enabled));
                            }
                            _ => (),
                        }
                    }
                }
                _ => (),
            };
        }
        self.state.reset();
    }

    pub fn write(&mut self, data: &str) {
        let string = String::from(data);
        for character in string.chars() {
            let code_point = character as u32;

            if code_point == 0x1b && self.state.sequence_type == SequenceType::None {
                self.state.sequence_type = SequenceType::ESC;
            } else if code_point == 0x9d && self.state.sequence_type == SequenceType::None {
                self.state.sequence_type = SequenceType::OSC;
                self.state.sequence = String::from("]");
            } else if character == '[' && self.state.sequence_type == SequenceType::ESC {
                self.state.sequence_type = SequenceType::ANSI;
                self.state.sequence = String::from("[");
            } else if character == ']' && self.state.sequence_type == SequenceType::ESC {
                self.state.sequence_type = SequenceType::OSC;
                self.state.sequence = String::from("]");
            } else if (character == '(' || character == ')') &&
                      self.state.sequence_type == SequenceType::ESC {
                self.state.sequence_type = SequenceType::OneChar;
            } else if self.state.sequence_type != SequenceType::None &&
                      self.state.sequence_type != SequenceType::ESC &&
                      (code_point == 0x1b || code_point == 0x9c || code_point == 0x07) {
                self.apply_sequence();
                if code_point == 0x1b {
                    self.state.sequence_type = SequenceType::ESC;
                }
            } else if self.state.sequence_type == SequenceType::ANSI && '\x40' <= character &&
                      character <= '\x7e' {
                self.state.sequence.push(character);
                self.apply_sequence();
            } else if self.state.sequence_type == SequenceType::ESC {
                if character == '\\' {
                    // ST
                    self.state.reset()
                }
            } else if self.state.sequence_type != SequenceType::None {
                self.state.sequence.push(character);
                if self.state.sequence_type == SequenceType::OneChar {
                    self.apply_sequence();
                }
            } else {
                match code_point {
                    0...2 => (),
                    3 => self.stack.push(Action::Interrupt),
                    4...6 => (),
                    7 => self.stack.push(Action::Bell),
                    8 => self.stack.push(Action::Backspace),
                    9 => self.stack.push(Action::Tab),
                    0xA => self.stack.push(Action::NewLine),
                    0xD => self.stack.push(Action::Return),
                    0x15 => self.stack.push(Action::DeleteLine),
                    0x17 => self.stack.push(Action::DeleteWord),
                    _ => {
                        self.stack.push(Action::Write(character.to_string()));
                    }
                }
            }
        }
    }
}

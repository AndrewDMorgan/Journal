#![allow(non_snake_case)]

use std::io::Write;
use vte::Perform;

// constants for tracking mouse scrolling
const SCROLL_SENSITIVITY: f64 = 0.05;
const SCROLL_LOG_TIME: f64 = 0.75;

#[derive(PartialEq, Eq, Debug, Default)]
pub enum KeyModifiers {
    Shift,
    #[default] Command,
    Option,
    Control,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum KeyCode {
    Delete,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Return,
    Escape,
}

#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub enum MouseEventType {
    #[default] Null,
    Left,
    Right,
    Middle,
    Down,
    Up,
}

#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub enum MouseState {
    Release,
    Press,
    Hold,
    #[default] Null,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct MouseEvent {
    pub eventType: MouseEventType,
    pub position: (u16, u16),
    pub state: MouseState,
}

#[derive(Default)]
pub struct KeyParser {
    pub keyModifiers: Vec <KeyModifiers>,
    pub keyEvents: std::collections::HashMap <KeyCode, bool>,
    pub charEvents: Vec <char>,
    pub inEscapeSeq: bool,
    pub bytes: usize,
    pub mouseEvent: Option <MouseEvent>,
    pub mouseModifiers: Vec <KeyModifiers>,
    pub lastPress: u128,
    pub scrollEvents: Vec <(std::time::SystemTime, i8)>,  // the sign is the direction
    pub scrollAccumulate: f64,
}

impl KeyParser {
    pub fn new () -> Self {
        KeyParser {
            keyEvents: std::collections::HashMap::from([
                (KeyCode::Delete, false),
                (KeyCode::Tab, false),
                (KeyCode::Left, false),
                (KeyCode::Right, false),
                (KeyCode::Up, false),
                (KeyCode::Down, false),
                (KeyCode::Return, false),
                (KeyCode::Escape, false),
            ]),
            keyModifiers: vec!(),
            charEvents: vec!(),
            inEscapeSeq: false,
            bytes: 0,
            mouseEvent: None,
            mouseModifiers: vec!(),
            lastPress: 0,
            scrollEvents: vec![],
            scrollAccumulate: 0.0,
        }
    }

    // tracking a log of scroll events to average them out over a duration of time
    fn Scroll (&mut self, sign: i8) {
        let time = std::time::SystemTime::now();
        if self.scrollAccumulate.is_sign_negative() != sign.is_negative(){
            self.scrollEvents.clear();  // so on sign flip it doesn't do weird things
        }
        self.scrollEvents.push((time, sign));
        self.UpdateScroll();
    }

    fn UpdateScroll(&mut self) {
        let time = std::time::SystemTime::now();
        let mut valid = vec![];
        let mut avg = 0.0;
        for (otherTime, otherSign) in &self.scrollEvents {
            // 0.000001 is the conversion rate from micro seconds to seconds
            let duration = time.duration_since(*otherTime).unwrap_or_default().as_secs_f64();
            if duration < SCROLL_LOG_TIME {
                avg += *otherSign as f64; valid.push((*otherTime, *otherSign));
            }
        }
        avg *= SCROLL_SENSITIVITY / SCROLL_LOG_TIME;
        self.scrollAccumulate = avg;
        self.scrollEvents = valid;
    }

    pub fn ClearEvents (&mut self) {
        self.charEvents.clear();
        self.keyModifiers.clear();
        self.mouseModifiers.clear();
        self.keyEvents.clear();
        self.inEscapeSeq = false;
        self.UpdateScroll();

        if let Some(event) = &mut self.mouseEvent {
            match event.state {
                MouseState::Press => {
                    event.state = MouseState::Hold;
                },
                MouseState::Hold if matches!(event.eventType, MouseEventType::Down | MouseEventType::Up) => {
                    event.state = MouseState::Release;
                },
                MouseState::Release => {
                    event.state = MouseState::Null;
                    event.eventType = MouseEventType::Null;
                },
                MouseState::Hold => {
                },
                _ => {},
            }
        }
    }

    pub fn ContainsChar (&self, chr: char) -> bool {
        self.charEvents.contains(&chr)
    }

    pub fn ContainsModifier (&self, modifier: &KeyModifiers) -> bool {
        self.keyModifiers.contains(modifier)
    }

    pub fn ContainsMouseModifier (&self, modifier: KeyModifiers) -> bool {
        self.mouseModifiers.contains(&modifier)
    }

    pub fn ContainsKeyCode (&self, key: KeyCode) -> bool {
        *self.keyEvents.get(&key).unwrap_or(&false)
    }

    fn HandleMouseEscapeCodes (&mut self, numbers: &[u16], c: char) {
        if let Some([byte, x, y]) = numbers.get(0..3) {
            let button = byte & 0b11; // Mask lowest 2 bits (button type)
            //println!("button: {}, numbers: {:?}", button, numbers);

            // adding key press modifiers
            if (byte & 32) != 0 {
                self.keyModifiers.push(KeyModifiers::Shift);
            }
            if (byte & 64) != 0 {
                self.keyModifiers.push(KeyModifiers::Option);
            }
            if (byte & 128) != 0 {
                self.keyModifiers.push(KeyModifiers::Control);
            }

            //println!("Code: {:?} / {}", numbers, c);

            let isScroll = (byte & 64) != 0;
            let eventType = match (isScroll, button) {
                (true, 0) => {
                    self.Scroll(-1i8);
                    MouseEventType::Up
                },
                (true, 1) => {
                    self.Scroll(1i8);
                    MouseEventType::Down
                },
                (false, 0) => MouseEventType::Left,
                (false, 1) => MouseEventType::Middle,
                (false, 2) => MouseEventType::Right,
                _ => MouseEventType::Null
            };

            if matches!(eventType, MouseEventType::Left) && numbers[0] == 4 {
                self.mouseModifiers.push(KeyModifiers::Shift);
            }

            self.CalculateMouseEventCode(eventType, (*x, *y), c);
        }
    }

    fn CalculateMouseEventCode (
        &mut self,
        eventType: MouseEventType,
        (x, y): (u16, u16),
        c: char
    ) {
        if let Some(event) = &mut self.mouseEvent {
            if matches!(eventType, MouseEventType::Left) &&
                event.position != (x, y) &&
                matches!(event.state, MouseState::Hold) &&
                c == 'M'
            {
                event.position = (x, y);
                return;
            }
        }

        self.mouseEvent = Some(MouseEvent {
            eventType,
            position: (x, y),
            state: {
                match c {
                    'M' => MouseState::Press,
                    'm' => MouseState::Release,
                    _ => MouseState::Null,
                }
            },
        });
    }

    fn HandleCustomEscapeCodes (&mut self, numbers: &[u16]) {
        match numbers[1] {
            2 => {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Shift);
            }
            3 => {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Option);
            }
            4 => {
                self.keyEvents.insert(KeyCode::Left, true);
                self.keyModifiers.push(KeyModifiers::Command);
            }
            5 => {
                self.keyEvents.insert(KeyCode::Right, true);
                self.keyModifiers.push(KeyModifiers::Command);
            }
            6 => {
                self.keyEvents.insert(KeyCode::Up, true);
                self.keyModifiers.push(KeyModifiers::Command);
            }
            7 => {
                self.keyEvents.insert(KeyCode::Down, true);
                self.keyModifiers.push(KeyModifiers::Command);
            }
            8 => {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Option);
                self.keyModifiers.push(KeyModifiers::Shift);
            }
            9 => {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Command);
            }
            10 => {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            }
            11 => {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('s');  // command + s
            }
            12 => {  // lrud
                self.keyEvents.insert(KeyCode::Left, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            }
            13 => {
                self.keyEvents.insert(KeyCode::Right, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            }
            14 => {
                self.keyEvents.insert(KeyCode::Up, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            }
            15 => {
                self.keyEvents.insert(KeyCode::Down, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            }
            16 => {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('c');
            }
            17 => {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('v');
            }
            18 => {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('x');
            }
            19 => {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('f');
            }
            20 => {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('z');
            }
            21 => {
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
                self.charEvents.push('z');
            }
            22 => {
                self.keyEvents.insert(KeyCode::Tab, true);
                self.keyModifiers.push(KeyModifiers::Option);
            }
            _ => {}
        }
    }

    fn HandleControlArrows (&mut self, _numbers: &[u16], c: char) {
        match c {
            'D' => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.keyEvents.insert(KeyCode::Left, true);
            },
            'C' => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.keyEvents.insert(KeyCode::Right, true);
            },
            'A' => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.keyEvents.insert(KeyCode::Up, true);
            },
            'B' => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.keyEvents.insert(KeyCode::Down, true);
            },
            _ => {}  // control + arrows
        }
    }

    fn HandleStandardEscapeCodes (&mut self, numbers: &Vec <u16>, c: char) {
        match c as u8 {
            0x5A => {
                self.keyEvents.insert(KeyCode::Tab, true);
                self.keyModifiers.push(KeyModifiers::Shift);
            },
            0x44 => {
                self.keyEvents.insert(KeyCode::Left, true);
                if *numbers == [1, 3] {
                    self.keyModifiers.push(KeyModifiers::Option);
                } else if *numbers == [1, 2] {
                    self.keyModifiers.push(KeyModifiers::Shift);
                } else if *numbers == [1, 4] {
                    self.keyModifiers.push(KeyModifiers::Option);
                    self.keyModifiers.push(KeyModifiers::Shift);
                }
            },
            0x43 => {
                self.keyEvents.insert(KeyCode::Right, true);
                if *numbers == [1, 3] {
                    self.keyModifiers.push(KeyModifiers::Option);
                } else if *numbers == [1, 2] {
                    self.keyModifiers.push(KeyModifiers::Shift);
                } else if *numbers == [1, 4] {
                    self.keyModifiers.push(KeyModifiers::Option);
                    self.keyModifiers.push(KeyModifiers::Shift);
                }
            },
            0x41 => {
                self.keyEvents.insert(KeyCode::Up, true);
                if *numbers == [1, 3] {
                    self.keyModifiers.push(KeyModifiers::Option);
                } else if *numbers == [1, 2] {
                    self.keyModifiers.push(KeyModifiers::Shift);
                } else if *numbers == [1, 4] {
                    self.keyModifiers.push(KeyModifiers::Option);
                    self.keyModifiers.push(KeyModifiers::Shift);
                }
            },
            0x42 => {
                self.keyEvents.insert(KeyCode::Down, true);
                if *numbers == [1, 3] {
                    self.keyModifiers.push(KeyModifiers::Option);
                } else if *numbers == [1, 2] {
                    self.keyModifiers.push(KeyModifiers::Shift);
                } else if *numbers == [1, 4] {
                    self.keyModifiers.push(KeyModifiers::Option);
                    self.keyModifiers.push(KeyModifiers::Shift);
                }
            },
            _ => {},
        }
    }

}

pub async fn enableMouseCapture() {
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(b"echo -e \"\x1B[?1006h\"");
    let _ = stdout.write_all(b"\x1B[?1000h"); // Enable basic mouse mode
    let _ = stdout.write_all(b"\x1B[?1003h"); // Enable all motion events
}

pub async fn disableMouseCapture() {
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(b"\x1B[?1000l"); // Disable mouse mode
    let _ = stdout.write_all(b"\x1B[?1003l"); // Disable motion events
}

impl KeyParser {
    pub fn SetPressTime (&mut self) {
        self.lastPress = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Time went backwards...")
            .as_millis();
    }
}

impl Perform for KeyParser {
    fn print(&mut self, chr: char) {
        //println!("char {}: '{}'", chr as u8, chr);
        if self.inEscapeSeq || self.bytes > 1 {
            match chr as u8 {
                17 => {
                    self.charEvents.push('w');
                    self.keyModifiers.push(KeyModifiers::Option);
                },
                _ => {}
            }

            return;
        }
        self.SetPressTime();

        if chr as u8 == 0x7F {
            self.keyEvents.insert(KeyCode::Delete, true);
            return;
        }
        if !(chr.is_ascii_graphic() || chr.is_whitespace()) {  return;  }
        //println!("char {}: '{}'", chr as u8, chr);
        self.charEvents.push(chr);
    }

    #[inline(always)]
    fn execute(&mut self, byte: u8) {
        self.SetPressTime();

        // control + ...
        // 3 = c; 22 = v; 26 = z; 6 = f; 1 = a; 24 = x; 19 = s; 21 = u; r = 18
        // left ^[[1;5D right ^[[1;5C up ^[[1;5A down ^[[1;5B
        // control u and control r and necessary for undo and redo bc/
        // control + key and control + shift + key don't send unique
        // escape codes for some odd reason

        match byte {
            0x1B => {
                self.inEscapeSeq = true;
            },
            0x0D => {  // return aka \n
                self.keyEvents.insert(KeyCode::Return, true);
            },
            0x09 => {
                self.keyEvents.insert(KeyCode::Tab, true);
            },// 3 = c; 22 = v; 26 = z; 6 = f; 1 = a; 24 = x; 19 = s; 21 = u; r = 18
            3 => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.charEvents.push('c');
            },
            22 => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.charEvents.push('v');
            },
            26 => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.charEvents.push('z');
            },
            6 => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.charEvents.push('f');
            },
            1 => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.charEvents.push('a');
            },
            24 => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.charEvents.push('x');
            },
            19 => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.charEvents.push('s');
            },
            21 => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.charEvents.push('u');
            },
            18 => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.charEvents.push('r');
            },
            0x08 => {
                self.keyModifiers.push(KeyModifiers::Control);
                self.keyEvents.insert(KeyCode::Delete, true);
            },
            10 => {
                self.charEvents.push('a');
                self.keyModifiers.push(KeyModifiers::Control);
            },
            _ => {},
        }
        //println!("byte {}: '{}'", byte, byte as char);
    }

    #[inline(always)]
    fn csi_dispatch(&mut self, params: &vte::Params, _: &[u8], _: bool, c: char) {
        self.inEscapeSeq = false;  // resetting the escape sequence
        self.SetPressTime();

        let numbers: Vec <u16> = params.iter().map(|p| p[0]).collect();

        // mouse handling
        if c == 'M' || c == 'm' {
            self.HandleMouseEscapeCodes(&numbers, c);
            return;
        }

        //for number in &numbers {println!("{}", number);}
        if c == '~' && numbers.len() == 2 && numbers[0] == 3 {  // this section is for custom escape codes
            self.HandleCustomEscapeCodes(&numbers);
        } else if numbers.len() == 2 && numbers[0] == 1 && numbers[1] == 5 {
            // control + ...
            // 3 = c; 22 = v; 26 = z; 6 = f; 1 = a; 24 = x; 19 = s; 21 = u; r = 18
            // left ^[[1;5D right ^[[1;5C up ^[[1;5A down ^[[1;5B
            // control u and control r and necessary for undo and redo bc/
            // control + key and control + shift + key don't send unique
            // escape codes for some odd reason
            self.HandleControlArrows(&numbers, c);
        } else {  // this checks existing escape codes of 1 parameter/ending code (they don't end with ~)
            self.HandleStandardEscapeCodes(&numbers, c);
        }
    }
}


use std::collections::VecDeque;
use std::iter::IntoIterator;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
}

impl Default for Key {
    fn default() -> Key {
        Key { code: KeyCode::Unknown(' '), alt: false, ctrl: false, shift: false }
    }
}

impl From<KeyCode> for Key {
    fn from(code: KeyCode) -> Key {
        Key { code: code, ..Default::default() }
    }
}

impl From<char> for Key {
    fn from(ch: char) -> Key {
        let mut shift = false;
        if ch.is_uppercase() {
            shift = true;
        }

        let keycode = KeyCode::from(ch); // Or unknown

        let mut key = Key::from(keycode);
        key.shift = shift;
        key
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyCode {
    D1, D2, D3, D4, D5, D6, D7, D8, D9, D0,
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    NumPad0, NumPad1, NumPad2, NumPad3, NumPad4,
    NumPad5, NumPad6, NumPad7, NumPad8, NumPad9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    Left,
    Right,
    Up,
    Down,

    Enter,
    Space,
    Esc,

    Tab,
    Backspace,
    Delete,
    Insert,

    Home,
    End,
    PageUp,
    PageDown,

    Unknown(char),
}

pub enum NumkeyType {
    Digit,
    NumPad,
    Function
}

impl From<char> for KeyCode {
    fn from(ch: char) -> KeyCode {
        if ch.is_numeric() {
            let numeric_ch = ch.to_digit(10).unwrap();
            return Key::numkey_code_from_digit(numeric_ch, NumkeyType::Digit);
        }
        // Lowercasing in Rust returns an iterator, making this match necessary.
        let ch_lower_iter = ch.to_lowercase().next();

        match ch_lower_iter {
            Some(ch_lower) => match ch_lower {
                'a' => KeyCode::A,
                'b' => KeyCode::B,
                'c' => KeyCode::C,
                'd' => KeyCode::D,
                'e' => KeyCode::E,
                'f' => KeyCode::F,
                'g' => KeyCode::G,
                'h' => KeyCode::H,
                'i' => KeyCode::I,
                'j' => KeyCode::J,
                'k' => KeyCode::K,
                'l' => KeyCode::L,
                'm' => KeyCode::M,
                'n' => KeyCode::N,
                'o' => KeyCode::O,
                'p' => KeyCode::P,
                'q' => KeyCode::Q,
                'r' => KeyCode::R,
                's' => KeyCode::S,
                't' => KeyCode::T,
                'u' => KeyCode::U,
                'v' => KeyCode::V,
                'w' => KeyCode::W,
                'x' => KeyCode::X,
                'y' => KeyCode::Y,
                'z' => KeyCode::Z,

                _   => KeyCode::Unknown(ch_lower),
            },
            None => KeyCode::Unknown(ch),
        }
    }
}

impl Key {
    pub fn numkey_code_from_digit(digit: u32, type_: NumkeyType) -> KeyCode {
        match type_ {
            NumkeyType::Digit => match digit {
                1 => KeyCode::D1,
                2 => KeyCode::D2,
                3 => KeyCode::D3,
                4 => KeyCode::D4,
                5 => KeyCode::D5,
                6 => KeyCode::D6,
                7 => KeyCode::D7,
                8 => KeyCode::D8,
                9 => KeyCode::D9,
                0 => KeyCode::D0,
                _ => KeyCode::Unknown(digit as u8 as char),
            },
            NumkeyType::NumPad => match digit {
                1 => KeyCode::NumPad1,
                2 => KeyCode::NumPad2,
                3 => KeyCode::NumPad3,
                4 => KeyCode::NumPad4,
                5 => KeyCode::NumPad5,
                6 => KeyCode::NumPad6,
                7 => KeyCode::NumPad7,
                8 => KeyCode::NumPad8,
                9 => KeyCode::NumPad9,
                0 => KeyCode::NumPad0,
                _ => KeyCode::Unknown(digit as u8 as char),
            },
            NumkeyType::Function => match digit {
                1 => KeyCode::F1,
                2 => KeyCode::F2,
                3 => KeyCode::F3,
                4 => KeyCode::F4,
                5 => KeyCode::F5,
                6 => KeyCode::F6,
                7 => KeyCode::F7,
                8 => KeyCode::F8,
                9 => KeyCode::F9,
                10 => KeyCode::F10,
                11 => KeyCode::F11,
                12 => KeyCode::F12,
                _ => KeyCode::Unknown(digit as u8 as char),
            }
        }
    }

}


#[derive(Debug)]
pub struct Keys {
    keys: VecDeque<Key>,
}


impl Keys {
    pub fn new() -> Self {
        Keys {
            keys: VecDeque::new(),
        }
    }

    /// Pop the `Key` from the beginning of the queue.
    pub fn pop(&mut self) -> Option<Key> {
        self.keys.pop_front()
    }

    /// Return true if any key matches the `predicate`.
    ///
    /// The keys will be checked in order they came in and the first
    /// one that matches will be taken out of the queue.
    pub fn matches<F>(&mut self, predicate: F) -> bool
        where F: Fn(Key) -> bool
    {

        let mut len = self.keys.len();
        let mut processed = 0;
        let mut found = false;
        while processed < len {
            match self.keys.pop_front() {
                Some(pressed_key) if !found && predicate(pressed_key) => {
                    len -= 1;
                    found = true;
                }
                Some(pressed_key) => {
                    self.keys.push_back(pressed_key);
                }
                None => return false
            }
            processed += 1;
        }
        return found;

    }

    /// Return true if any key has the specified key code.
    ///
    /// The keys will be checked in order they came in and the first
    /// one that matches will be taken out of the queue.
    pub fn matches_code(&mut self, key_code: KeyCode) -> bool {
        self.matches(|k| k.code == key_code)
    }

    pub fn extend<T: IntoIterator<Item=Key>>(&mut self, iterator: T) {
        self.keys.extend(iterator)
    }
}

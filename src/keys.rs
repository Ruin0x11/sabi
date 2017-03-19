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
        Key { code: KeyCode::NoneKey, alt: false, ctrl: false, shift: false }
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

        let keycode = keycode_from_char(ch).unwrap_or(KeyCode::NoneKey);

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

    NoneKey,
}

pub enum NumkeyType {
    Digit,
    NumPad,
    Function
}

pub fn numkey_code_from_digit(digit: u32, type_: NumkeyType) -> Option<KeyCode> {
    match type_ {
        NumkeyType::Digit => match digit {
            1 => Some(KeyCode::D1),
            2 => Some(KeyCode::D2),
            3 => Some(KeyCode::D3),
            4 => Some(KeyCode::D4),
            5 => Some(KeyCode::D5),
            6 => Some(KeyCode::D6),
            7 => Some(KeyCode::D7),
            8 => Some(KeyCode::D8),
            9 => Some(KeyCode::D9),
            0 => Some(KeyCode::D0),
            _ => None,
        },
        NumkeyType::NumPad => match digit {
            1 => Some(KeyCode::NumPad1),
            2 => Some(KeyCode::NumPad2),
            3 => Some(KeyCode::NumPad3),
            4 => Some(KeyCode::NumPad4),
            5 => Some(KeyCode::NumPad5),
            6 => Some(KeyCode::NumPad6),
            7 => Some(KeyCode::NumPad7),
            8 => Some(KeyCode::NumPad8),
            9 => Some(KeyCode::NumPad9),
            0 => Some(KeyCode::NumPad0),
            _ => None,
        },
        NumkeyType::Function => match digit {
            1 => Some(KeyCode::F1),
            2 => Some(KeyCode::F2),
            3 => Some(KeyCode::F3),
            4 => Some(KeyCode::F4),
            5 => Some(KeyCode::F5),
            6 => Some(KeyCode::F6),
            7 => Some(KeyCode::F7),
            8 => Some(KeyCode::F8),
            9 => Some(KeyCode::F9),
            10 => Some(KeyCode::F10),
            11 => Some(KeyCode::F11),
            12 => Some(KeyCode::F12),
            _ => None,
        }
    }

}

pub fn keycode_from_char(ch: char) -> Option<KeyCode> {
    if ch.is_numeric() {
        let numeric_ch = ch.to_digit(10).unwrap();
        return numkey_code_from_digit(numeric_ch, NumkeyType::Digit);
    }
    // Lowercasing in Rust returns an iterator, making this match necessary.
    let ch_lower_iter = ch.to_lowercase().next();

    match ch_lower_iter {
        Some(ch_lower) => match ch_lower {
            'a' => Some(KeyCode::A),
            'b' => Some(KeyCode::B),
            'c' => Some(KeyCode::C),
            'd' => Some(KeyCode::D),
            'e' => Some(KeyCode::E),
            'f' => Some(KeyCode::F),
            'g' => Some(KeyCode::G),
            'h' => Some(KeyCode::H),
            'i' => Some(KeyCode::I),
            'j' => Some(KeyCode::J),
            'k' => Some(KeyCode::K),
            'l' => Some(KeyCode::L),
            'm' => Some(KeyCode::M),
            'n' => Some(KeyCode::N),
            'o' => Some(KeyCode::O),
            'p' => Some(KeyCode::P),
            'q' => Some(KeyCode::Q),
            'r' => Some(KeyCode::R),
            's' => Some(KeyCode::S),
            't' => Some(KeyCode::T),
            'u' => Some(KeyCode::U),
            'v' => Some(KeyCode::V),
            'w' => Some(KeyCode::W),
            'x' => Some(KeyCode::X),
            'y' => Some(KeyCode::Y),
            'z' => Some(KeyCode::Z),

            _   => None
        },
        None => None
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

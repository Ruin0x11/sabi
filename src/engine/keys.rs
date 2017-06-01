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
    Escape,

    Delete,
    Insert,

    Home,
    End,
    PageUp,
    PageDown,

    GreaterThan,
    LessThan,

    Comma,
    Period,

    Unknown(char),
}

impl From<char> for KeyCode {
    fn from(ch: char) -> KeyCode {
        if ch.is_numeric() {
            let numeric_ch = ch.to_digit(10).unwrap();
            return Key::numkey_code_from_digit(numeric_ch);
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

                '>' => KeyCode::GreaterThan,
                '<' => KeyCode::LessThan,

                _   => KeyCode::Unknown(ch_lower),
            },
            None => KeyCode::Unknown(ch),
        }
    }
}

impl Key {
    pub fn numkey_code_from_digit(digit: u32) -> KeyCode {
        match digit {
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
        }
    }
}

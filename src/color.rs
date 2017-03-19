use std::f64;
use std::cmp::min;

pub type Color = RGB;

#[derive (PartialEq, Eq, Debug, Clone)]
pub enum Color16 {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Default,
}

const LOOKUP_16 : [(RGB, Color16); 8] = [(BLACK,   Color16::Black),
                                         (RED,     Color16::Red),
                                         (GREEN,   Color16::Green),
                                         (YELLOW,  Color16::Yellow),
                                         (BLUE,    Color16::Blue),
                                         (MAGENTA, Color16::Magenta),
                                         (CYAN,    Color16::Cyan),
                                         (WHITE,   Color16::White),
];

impl From<Color256> for Color16 {
    fn from(c: Color256) -> Color16 {
        Color16::from(c.to_rgb())
    }
}

impl From<RGB> for Color16 {
    fn from(c: RGB) -> Color16 {
        // Quantize to the nearest 3-bit color
        let mut my_color = Color16::Default;
        let mut dist = f64::MAX;
        for &(ref rgb, ref color) in LOOKUP_16.iter() {
            let newdist = c.dist(rgb);
            if newdist < dist {
                dist = newdist;
                my_color = color.clone();
            }
        }
        my_color
    }
}

pub struct Color256(u8);

impl Color256 {
    pub fn to_rgb(&self) -> RGB {
        let newcol = self.to_color216();
        let factor = 255 / 6;

        RGB::new(newcol.r * factor, newcol.g * factor, newcol.b * factor)
    }
    pub fn to_color216(&self) -> Color216 {
        match self.0 {
            7 => Color256(255).to_color216(),
            15 => Color216::new(5, 5, 5),
            16...231 => {
                let c = self.0 - 16;
                let b = c % 6;
                let c = (c - b) / 6;
                let g = c % 6;
                let r = (c - g) / 6;
                Color216::new(r, g, b)
            },
            232...255 => {
                let c = (self.0 - 232) / 4;
                Color216::new(c, c, c)
            }
            _ => panic!("Unimplemented color {}", self.0),
        }
    }

    pub fn mix(&self, color : Color256, s : u8) -> Color256 {
        assert!(s < 6);
        let s_rgb = self.to_color216();
        let c_rgb = color.to_color216();
        Color216::new(
            (s_rgb.r * (5 - s) + c_rgb.r * s) / 5,
            (s_rgb.g * (5 - s) + c_rgb.g * s) / 5,
            (s_rgb.b * (5 - s) + c_rgb.b * s) / 5,
            ).into()
    }
}

impl From<Color216> for Color256 {
    fn from(rgb : Color216) -> Self {
        Color256(rgb.to_u8())
    }
}

impl From<u8> for Color256 {
    fn from(u : u8) -> Self {
        Color256(u)
    }
}

pub struct Color216 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color216 {
    pub fn new(r : u8, g : u8, b : u8) -> Self {
        assert!(r < 6);
        assert!(g < 6);
        assert!(b < 6);

        Color216 { r: r, g: g, b: b }
    }

    pub fn to_u8(&self) -> u8 {
        16 + self.r * 36 + self.g * 6 + self.b
    }

}

pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r : u8, g : u8, b : u8) -> Self {
        RGB { r: r, g: g, b: b }
    }

    pub fn dist(&self, other: &RGB) -> f64 {
        let sq = |num| (num as f64) * (num as f64);
        let sr = sq(self.r as i16 - other.r as i16);
        let sg = sq(self.g as i16 - other.g as i16);
        let sb = sq(self.b as i16 - other.b as i16);
        (sr + sg + sb).sqrt()
    }
}

pub const BLACK:   Color =  Color{ r: 0,    g: 0,   b: 0 };   
pub const RED:     Color =  Color{ r: 255,  g: 0,   b: 0 };   
pub const GREEN:   Color =  Color{ r: 0,    g: 255, b: 0 };   
pub const YELLOW:  Color =  Color{ r: 255,  g: 255, b: 0 };   
pub const BLUE:    Color =  Color{ r: 0,    g: 0,   b: 255 }; 
pub const MAGENTA: Color =  Color{ r: 255,  g: 0,   b: 255 }; 
pub const CYAN:    Color =  Color{ r: 0,    g: 255, b: 255 }; 
pub const WHITE:   Color =  Color{ r: 255,  g: 255, b: 255 }; 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_16() {
        assert_eq!(Color16::from(BLACK),   Color16::Black);
        assert_eq!(Color16::from(RED),   Color16::Red);
        assert_eq!(Color16::from(GREEN),   Color16::Green);
        assert_eq!(Color16::from(YELLOW),   Color16::Yellow);
        assert_eq!(Color16::from(BLUE), Color16::Blue);
        assert_eq!(Color16::from(MAGENTA), Color16::Magenta);
        assert_eq!(Color16::from(CYAN), Color16::Cyan);
        assert_eq!(Color16::from(WHITE), Color16::White);
    }
}

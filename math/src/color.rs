#[derive(Clone, PartialOrd, PartialEq, Debug, Copy)]
pub enum PresetColors {
    RED,
    BLUE,
    GREEN,
    MAGENTA,
    YELLOW,
    TEAL,
    WHITE,
    GREY,
    BLACK,
    TRANSPARENT,
    Other(u8, u8, u8, u8)
}

impl PresetColors {
    fn to_tuple(self) -> (u8, u8, u8, u8) {
        match self {
            PresetColors::RED => (255, 0, 0, 255),
            PresetColors::GREEN => (0, 255, 0, 255),
            PresetColors::BLUE => (0, 0, 255, 255),
            PresetColors::YELLOW => (255, 255, 0, 255),
            PresetColors::MAGENTA => (255, 0, 255, 255),
            PresetColors::TEAL => (0, 255, 255, 255),
            PresetColors::GREY => (128, 128, 128, 255),
            PresetColors::BLACK => (0, 0, 0, 255),
            PresetColors::WHITE => (255, 255, 255, 255),
            PresetColors::TRANSPARENT => (0,0,0,0),
            PresetColors::Other(r, g, b, a) => { (r, g, b, a) }
        }
    }


}

impl From<PresetColors> for (f32, f32,f32) {
    fn from(color: PresetColors) -> Self {
        let color = color.to_tuple();
        (color.0 as f32 / 255., color.1 as f32 / 255., color.2 as f32 / 255.)
    }
}
impl From<PresetColors> for (f32, f32, f32, f32) {
    fn from(color: PresetColors) -> Self {
        let color = color.to_tuple();
        (color.0 as f32 / 255., color.1 as f32 / 255., color.2 as f32 / 255., color.3 as f32 / 255.)
    }
}
impl From<PresetColors> for [f32; 3] {
    fn from(color: PresetColors) -> Self {
        let color = color.to_tuple();
        [color.0 as f32 / 255., color.1 as f32 / 255., color.2 as f32 / 255.]
    }
}
impl From<PresetColors> for [f32; 4] {
    fn from(color: PresetColors) -> Self {
        let color = color.to_tuple();
        [color.0 as f32 / 255., color.1 as f32 / 255., color.2 as f32 / 255., color.3 as f32 / 255.]
    }
}

#[derive(Clone, Copy)]
pub struct RGB {
    pub r: f32,
    pub g: f32, 
    pub b: f32,
}
impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0
        }
    }

    pub fn new_f32(r: f32, g: f32, b: f32) -> Self {
        Self {
            r, g, b
        }
    }
}

impl From<RGB> for [f32;3] {
    fn from(color: RGB) -> Self {
        [color.r, color.g, color.b]
    }
}
impl From<RGB> for (f32, f32, f32) {
    fn from(color: RGB) -> Self {
        (color.r, color.g, color.b)
    }
}

pub struct RGBA {
    pub r: f32,
    pub g: f32, 
    pub b: f32,
    pub a: f32
}
impl RGBA {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0
        }
    }

    pub fn new_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r, g, b, a
        }
    }
}


#[derive(Clone, Copy)]
pub struct HSL {
    h: i16,
    s: f32,
    l: f32,
}
impl HSL {
    pub fn new(h: u16, s: f32, l: f32) -> Self {
        Self {
            h: h as i16,s, l
        }
    }

    pub fn to_rgb(self) -> RGB {
        let c = (1. - (2.*self.l - 1.).abs() ) * self.s;
        let x = c * ( 1. - ((self.h-60)%2 - 1).abs() as f32);
        let m = self.l - c/2.;
        let h = self.h % 360;
        let (r,g,b) =  
            if h < 60 {
                (c,x,0.)
            } else if h < 120 {
                (x,c,0.)
            } else if h < 180 {
                (0., c, x)
            } else if h < 240 {
                (0., x, c)
            } else if h < 300 {
                (x, 0., c)
            } else {
                (c, 0., x)
            };
        RGB::new_f32(r, g, b)
    }
}

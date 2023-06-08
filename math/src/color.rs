#[derive(Clone, PartialOrd, PartialEq, Debug, Copy)]
pub enum Colors {
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

impl Colors {
    fn to_tuple(self) -> (u8, u8, u8, u8) {
        match self {
            Colors::RED => (255, 0, 0, 255),
            Colors::GREEN => (0, 255, 0, 255),
            Colors::BLUE => (0, 0, 255, 255),
            Colors::YELLOW => (255, 255, 0, 255),
            Colors::MAGENTA => (255, 0, 255, 255),
            Colors::TEAL => (0, 255, 255, 255),
            Colors::GREY => (128, 128, 128, 255),
            Colors::BLACK => (0, 0, 0, 255),
            Colors::WHITE => (255, 255, 255, 255),
            Colors::TRANSPARENT => (0,0,0,0),
            Colors::Other(r, g, b, a) => { (r, g, b, a) }
        }
    }


}

impl From<Colors> for (f32, f32,f32) {
    fn from(color: Colors) -> Self {
        let color = color.to_tuple();
        (color.0 as f32 / 255., color.1 as f32 / 255., color.2 as f32 / 255.)
    }
}
impl From<Colors> for (f32, f32, f32, f32) {
    fn from(color: Colors) -> Self {
        let color = color.to_tuple();
        (color.0 as f32 / 255., color.1 as f32 / 255., color.2 as f32 / 255., color.3 as f32 / 255.)
    }
}
impl From<Colors> for [f32; 3] {
    fn from(color: Colors) -> Self {
        let color = color.to_tuple();
        [color.0 as f32 / 255., color.1 as f32 / 255., color.2 as f32 / 255.]
    }
}
impl From<Colors> for [f32; 4] {
    fn from(color: Colors) -> Self {
        let color = color.to_tuple();
        [color.0 as f32 / 255., color.1 as f32 / 255., color.2 as f32 / 255., color.3 as f32 / 255.]
    }
}

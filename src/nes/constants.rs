pub const SCREEN_RES_X: u32 = 256;
pub const SCREEN_RES_Y: u32 = 240;

pub const SCREEN_WIDTH: u32 = 512;
pub const SCREEN_HEIGHT: u32 = 480;

pub const PATTERN_TABLE_X: u32 = 128;
pub const PATTERN_TABLE_Y: u32 = 128;

#[derive(Copy, Clone, Debug)]
pub struct Color {
  pub val: [u8; 3]
}

pub const COLORS: [Color; 64] = [
  Color { val: [84, 84, 84] },
  Color { val: [0, 30, 116] },
  Color { val: [8, 16, 144] },
  Color { val: [48, 0, 136] },
  Color { val: [68, 0, 100] },
  Color { val: [92, 0, 48] },
  Color { val: [84, 4, 0] },
  Color { val: [60, 24, 0] },
  Color { val: [32, 42, 0] },
  Color { val: [8, 58, 0] },
  Color { val: [0, 64, 0] },
  Color { val: [0, 60, 0] },
  Color { val: [0, 50, 60] },
  Color { val: [0, 0, 0] },
  Color { val: [0, 0, 0] },
  Color { val: [0, 0, 0] },
  Color { val: [152, 150, 152] },
  Color { val: [8, 76, 196] },
  Color { val: [48, 50, 236] },
  Color { val: [92, 30, 228] },
  Color { val: [136, 20, 176] },
  Color { val: [160, 20, 100] },
  Color { val: [152, 34, 32] },
  Color { val: [120, 60, 0] },
  Color { val: [84, 90, 0] },
  Color { val: [40, 114, 0] },
  Color { val: [8, 124, 0] },
  Color { val: [0, 118, 40] },
  Color { val: [0, 102, 120] },
  Color { val: [0, 0, 0] },
  Color { val: [0, 0, 0] },
  Color { val: [0, 0, 0] },
  Color { val: [236, 238, 236] },
  Color { val: [76, 154, 236] },
  Color { val: [120, 124, 236] },
  Color { val: [176, 98, 236] },
  Color { val: [228, 84, 236] },
  Color { val: [236, 88, 180] },
  Color { val: [236, 106, 100] },
  Color { val: [212, 136, 32] },
  Color { val: [160, 170, 0] },
  Color { val: [116, 196, 0] },
  Color { val: [76, 208, 32] },
  Color { val: [56, 204, 108] },
  Color { val: [56, 180, 204] },
  Color { val: [60, 60, 60] },
  Color { val: [0, 0, 0] },
  Color { val: [0, 0, 0] },
  Color { val: [236, 238, 236] },
  Color { val: [168, 204, 236] },
  Color { val: [188, 188, 236] },
  Color { val: [212, 178, 236] },
  Color { val: [236, 174, 236] },
  Color { val: [236, 174, 212] },
  Color { val: [236, 180, 176] },
  Color { val: [228, 196, 144] },
  Color { val: [204, 210, 120] },
  Color { val: [180, 222, 120] },
  Color { val: [168, 226, 144] },
  Color { val: [152, 226, 180] },
  Color { val: [160, 214, 228] },
  Color { val: [160, 162, 160] },
  Color { val: [0, 0, 0] },
  Color { val: [0, 0, 0] },
];

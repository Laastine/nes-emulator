use bitflags::bitflags;


bitflags! {
    pub struct PpuCtrlFlags: u8 {
        const NAMETABLE_X = 1 << 0;
        const NAMETABLE_Y = 1 << 1;
        const VRAM_ADDR_INCREMENT_MODE = 1 << 2;
        const PATTERN_SPRITE_TABLE_ADDR = 1 << 3;
        const PATTERN_BACKGROUND_TABLE_ADDR = 1 << 4;
        const SPRITE_SIZE = 1 << 5;
        const SLAVE_MODE = 1 << 6;
        const ENABLE_NMI = 1 << 7;
    }
}

bitflags! {
    pub struct PpuMaskFlags: u8 {
        const GRAYSCALE = 1 << 0;
        const SHOW_BACKGROUND_IN_LEFT_MARGIN = 1 << 1;
        const SHOW_SPRITES_IN_LEFT_MARGIN = 1 << 2;
        const SHOW_BACKGROUND = 1 << 3;
        const SHOW_SPRITES = 1 << 4;
        const EMPHASIZE_RED = 1 << 5;
        const EMPHASIZE_GREEN = 1 << 6;
        const EMPHASIZE_BLUE = 1 << 7;
    }
}

bitflags! {
    pub struct PpuStatusFlags: u8 {
        const SPRITE_OVERFLOW = 1 << 5;
        const SPRITE_ZERO_HIT = 1 << 6;
        const VERTICAL_BLANK_STARTED = 1 << 7;
    }
}

// https://wiki.nesdev.com/w/index.php/PPU_scrolling
bitflags! {
  pub struct ScrollRegister: u16 {
      const COARSE_X = 1 << 0;
      const COARSE_Y = 1 << 1;
      const NAMETABLE_X = 1 << 2;
      const NAMETABLE_Y = 1 << 3;
      const FINE_Y = 1 << 4;
      const UNUSED = 1 << 5;
  }
}

#[derive(Copy, Clone)]
pub struct Registers {
  pub ctrl_flags: PpuCtrlFlags,
  pub mask_flags: PpuMaskFlags,
  pub status_flags: PpuStatusFlags,
  pub vram_addr: ScrollRegister,
  pub tram_addr: ScrollRegister,
  pub address_latch: u8,
  pub ppu_data_buffer: u8,
  pub fine_x: u8,
}

impl Registers {
  pub fn new() -> Registers {
    Registers {
      ctrl_flags: PpuCtrlFlags::from_bits_truncate(0x00),
      mask_flags: PpuMaskFlags::from_bits_truncate(0x00),
      status_flags: PpuStatusFlags::from_bits_truncate(0x00),
      vram_addr: ScrollRegister::from_bits_truncate(0x00),
      tram_addr: ScrollRegister::from_bits_truncate(0x00),
      address_latch: 0x00,
      ppu_data_buffer: 0x00,
      fine_x: 0x00,
    }
  }
}

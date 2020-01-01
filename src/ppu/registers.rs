use bitflags::bitflags;
use std::convert::TryFrom;
use crate::cartridge::rom::Mirroring;


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
      const COARSE_X = 1 << 4;
      const COARSE_Y = 1 << 9;
      const NAMETABLE_X = 1 << 10;
      const NAMETABLE_Y = 1 << 11;
      const FINE_Y = 1 << 14;
      const UNUSED = 1 << 15;
  }
}

#[derive(Copy, Clone)]
pub struct Registers {
  pub ctrl_flags: PpuCtrlFlags,
  pub mask_flags: PpuMaskFlags,
  pub status_flags: PpuStatusFlags,
  pub vram_addr: ScrollRegister,
  pub tram_addr: ScrollRegister,
  pub table_palette: [u8; 32],
  pub table_name: [[u8; 1024]; 2],
  pub address_latch: u8,
  pub ppu_data_buffer: u8,
  pub fine_x: u8,
  pub fine_y: u8,
  mirror_mode: Mirroring,
}

impl Registers {
  pub fn new(mirror_mode: Mirroring) -> Registers {
    Registers {
      ctrl_flags: PpuCtrlFlags::from_bits_truncate(0x00),
      mask_flags: PpuMaskFlags::from_bits_truncate(0x00),
      status_flags: PpuStatusFlags::from_bits_truncate(0x00),
      vram_addr: ScrollRegister::from_bits_truncate(0x00),
      tram_addr: ScrollRegister::from_bits_truncate(0x00),
      table_palette: [0; 32],
      table_name: [[0; 1024]; 2],
      address_latch: 0x00,
      ppu_data_buffer: 0x00,
      fine_x: 0x00,
      fine_y: 0x00,
      mirror_mode,
    }
  }

  pub fn ppu_read(&mut self, address: u16) -> u8 {
    let mut addr = address & 0x3FFF;

    match addr {
      0x0000..=0x1FFF => {
        let first_idx = usize::try_from((addr & 0x1000) >> 12).unwrap();
        let second_idx = usize::try_from(addr & 0x0FFF).unwrap();
        self.table_name[first_idx][second_idx]
      }
      0x2000..=0x3EFF => {
        addr &= 0x0FFF;
        let idx = usize::try_from(addr & 0x03FF).unwrap();
        match self.mirror_mode {
          Mirroring::Vertical => {
            match addr {
              0x0000..=0x03FF => self.table_name[0][idx],
              0x0400..=0x07FF => self.table_name[1][idx],
              0x0800..=0x0BFF => self.table_name[0][idx],
              0x0C00..=0x0FFF => self.table_name[1][idx],
              _ => panic!("Unknown vertical mode table address"),
            }
          }
          Mirroring::Horizontal => {
            match addr {
              0x0000..=0x03FF => self.table_name[0][idx],
              0x0400..=0x07FF => self.table_name[0][idx],
              0x0800..=0x0BFF => self.table_name[1][idx],
              0x0C00..=0x0FFF => self.table_name[1][idx],
              _ => panic!("Unknown horizontal mode table address"),
            }
          }
        }
      }
      0x3F00..=0x3FFF => {
        addr &= 0x001F;
        let idx = match addr {
          0x0010 => 0x0000,
          0x0014 => 0x0004,
          0x0018 => 0x0008,
          0x001C => 0x000C,
          _ => panic!("No palette idx found")
        };
        self.table_palette[idx] & (if self.mask_flags.contains(PpuMaskFlags::GRAYSCALE) { 0x30 } else { 0x3F })
      }
      _ => panic!("Address {} not in range", addr)
    }
  }

  pub fn ppu_write(&mut self, address: u16, data: u8) {
    let mut addr = address & 0x3FFF;

    match addr {
      0x0000..=0x1FFF => {
        let first_idx = usize::try_from((addr & 0x1000) >> 12).unwrap();
        let second_idx = usize::try_from(addr & 0x0FFF).unwrap();
        self.table_name[first_idx][second_idx] = data;
      }
      0x2000..=0x3EFF => {
        addr &= 0x0FFF;
        let idx = usize::try_from(addr & 0x03FF).unwrap();
        match self.mirror_mode {
          Mirroring::Vertical => {
            match addr {
              0x0000..=0x03FF => self.table_name[0][idx] = data,
              0x0400..=0x07FF => self.table_name[1][idx] = data,
              0x0800..=0x0BFF => self.table_name[0][idx] = data,
              0x0C00..=0x0FFF => self.table_name[1][idx] = data,
              _ => panic!("Unknown vertical mode table address"),
            }
          }
          Mirroring::Horizontal => {
            match addr {
              0x0000..=0x03FF => self.table_name[0][idx] = data,
              0x0400..=0x07FF => self.table_name[0][idx] = data,
              0x0800..=0x0BFF => self.table_name[1][idx] = data,
              0x0C00..=0x0FFF => self.table_name[1][idx] = data,
              _ => panic!("Unknown horizontal mode table address"),
            }
          }
        }
      }
      0x3F00..=0x3FFF => {
        addr &= 0x001F;
        let idx = match addr {
          0x0010 => 0x0000,
          0x0014 => 0x0004,
          0x0018 => 0x0008,
          0x001C => 0x000C,
          _ => panic!("No palette idx found")
        };
        self.table_palette[idx] = data;
      }
      _ => panic!("Address {} not in range", addr)
    }
  }
}

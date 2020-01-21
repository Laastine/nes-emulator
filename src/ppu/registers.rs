use std::cell::{RefCell, RefMut};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::cartridge::Cartridge;
use crate::cartridge::rom_reading::Mirroring;
use std::fs::OpenOptions;
use std::io::Write;

bitfield! {
  #[derive(Copy, Clone, PartialEq)]
  pub struct PpuCtrlFlags(u8); impl Debug;
    pub u8, nametable_x, _: 0;
    pub u8, nametable_y, _: 1;
    pub u8, vram_addr_increment_mode, _: 2;
    pub u8, pattern_sprite_table_addr, _: 3;
    pub u8, pattern_background, _: 4;
    pub u8, sprite_size, _: 5;
    pub u8, slave_mode, _: 6;
    pub u8, enable_nmi, _: 7;
}

bitfield! {
  #[derive(Copy, Clone, PartialEq)]
  pub struct PpuMaskFlags(u8); impl Debug;
    pub u8, grayscale, _: 0;
    pub u8, show_background_in_left_margin, _: 1;
    pub u8, show_sprites_in_left_margin, _: 2;
    pub u8, show_background, _: 3;
    pub u8, show_sprites, _: 4;
    pub u8, emphasize_red, _: 5;
    pub u8, emphasize_green, _: 6;
    pub u8, emphasize_blue, _: 7;
}

bitfield! {
  #[derive(Copy, Clone, PartialEq)]
  pub struct PpuStatusFlags(u8); impl Debug;
    pub u8, sprite_overflow, set_sprite_overflow:               5;
    pub u8, sprite_zero_hit, set_sprite_zero_hit:               6;
    pub u8, vertical_blank,  set_vertical_blank:                7;
}

// https://wiki.nesdev.com/w/index.php/PPU_scrolling
bitfield! {
  #[derive(Copy, Clone, PartialEq)]
  pub struct ScrollRegister(u16); impl Debug;
    pub u8,    coarse_x,     set_coarse_x:      4,  0;
    pub u8,    coarse_y,     set_coarse_y:      9,  5;
    pub u8,    nametable_x,  set_nametable_x:   10;
    pub u8,    nametable_y,  set_nametable_y:   11;
    pub u8,    fine_y,       set_fine_y:        14, 12;
    pub u8,    _unused,       _:                15, 15;
    pub u8,    hi_byte,      set_hi_byte:       13, 8;
    pub u8,    lo_byte,      set_lo_byte:       7,  0;
}

#[derive(Clone)]
pub struct Registers {
  pub ctrl_flags: PpuCtrlFlags,
  pub mask_flags: PpuMaskFlags,
  pub status_flags: PpuStatusFlags,
  pub vram_addr: ScrollRegister,
  pub tram_addr: ScrollRegister,
  pub palette_table: [u8; 32],
  pub table_pattern: [[u8; 4096]; 2],
  pub name_table: [[u8; 1024]; 2],
  pub address_latch: bool,
  pub ppu_data_buffer: u8,
  pub fine_x: u8,
  pub cartridge: Rc<RefCell<Cartridge>>,
}

impl Registers {
  pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Registers {
    Registers {
      ctrl_flags: PpuCtrlFlags(0x00),
      mask_flags: PpuMaskFlags(0x00),
      status_flags: PpuStatusFlags(0x00),
      vram_addr: ScrollRegister(0x00),
      tram_addr: ScrollRegister(0x00),
      palette_table: [0; 32],
      table_pattern: [[0; 4096]; 2],
      name_table: [[0; 1024]; 2],
      address_latch: false,
      ppu_data_buffer: 0x00,
      fine_x: 0x00,
      cartridge,
    }
  }

  fn get_mut_cartridge(&mut self) -> RefMut<Cartridge> {
    self.cartridge.borrow_mut()
  }

  pub fn ppu_read(&mut self, address: u16) -> u8 {
    let mut addr = address & 0x3FFF;

    let (is_address_in_range, mapped_addr) = self.get_mut_cartridge().mapper.mapped_read_ppu_u8(addr);
    let res = if is_address_in_range {
      self.get_mut_cartridge().rom.chr_rom[mapped_addr]
    } else if (0x0000..=0x1FFF).contains(&addr) {
      let first_idx = usize::try_from((addr & 0x1000) >> 12).unwrap();
      let second_idx = usize::try_from(addr & 0x0FFF).unwrap();
      self.table_pattern[first_idx][second_idx]
    } else if (0x2000..=0x3EFF).contains(&addr) {
      addr &= 0x0FFF;
      let idx = usize::try_from(addr & 0x03FF).unwrap();
      let mirror_mode = self.get_mut_cartridge().get_mirror_mode();
      match mirror_mode {
        Mirroring::Vertical => {
          match addr {
            0x0000..=0x03FF => self.name_table[0][idx],
            0x0400..=0x07FF => self.name_table[1][idx],
            0x0800..=0x0BFF => self.name_table[0][idx],
            0x0C00..=0x0FFF => self.name_table[1][idx],
            _ => panic!("Unknown vertical mode table address"),
          }
        }
        Mirroring::Horizontal => {
          match addr {
            0x0000..=0x03FF => self.name_table[0][idx],
            0x0400..=0x07FF => self.name_table[0][idx],
            0x0800..=0x0BFF => self.name_table[1][idx],
            0x0C00..=0x0FFF => self.name_table[1][idx],
            _ => panic!("Unknown horizontal mode table address"),
          }
        }
      }
    }
    else if (0x3F00..=0x3FFF).contains(&addr) {
      addr &= 0x001F;
      let idx: usize = match addr {
        0x0010 | 0x0014 | 0x0018 | 0x001C => usize::try_from(addr).unwrap() - 0x10,
        _ => addr.into()
      };
      self.palette_table[idx] & if self.mask_flags.grayscale() { 0x30 } else { 0x3F }
    } else {
      0
    };
    res
  }

  pub fn ppu_write(&mut self, address: u16, data: u8) {
    let mut addr = address & 0x3FFF;

    let (is_address_in_range, mapped_addr) = self.get_mut_cartridge().mapper.mapped_write_ppu_u8(addr);
    if is_address_in_range {
      self.get_mut_cartridge().rom.chr_rom[mapped_addr] = data;
    } else if (0x0000..=0x1FFF).contains(&addr) {
      let first_idx = usize::try_from((addr & 0x1000) >> 12).unwrap();
      let second_idx = usize::try_from(addr & 0x0FFF).unwrap();
      self.table_pattern[first_idx][second_idx] = data;
    } else if (0x2000..=0x3EFF).contains(&addr) {
      addr &= 0x0FFF;
      let idx = usize::try_from(addr & 0x03FF).unwrap();
      let mirror_mode = self.get_mut_cartridge().get_mirror_mode();
      match mirror_mode {
        Mirroring::Vertical => {
          match addr {
            0x0000..=0x03FF => self.name_table[0][idx] = data,
            0x0400..=0x07FF => self.name_table[1][idx] = data,
            0x0800..=0x0BFF => self.name_table[0][idx] = data,
            0x0C00..=0x0FFF => self.name_table[1][idx] = data,
            _ => panic!("Unknown vertical mode table address"),
          }
        }
        Mirroring::Horizontal => {
          match addr {
            0x0000..=0x03FF => self.name_table[0][idx] = data,
            0x0400..=0x07FF => self.name_table[0][idx] = data,
            0x0800..=0x0BFF => self.name_table[1][idx] = data,
            0x0C00..=0x0FFF => self.name_table[1][idx] = data,
            _ => panic!("Unknown horizontal mode table address"),
          }
        }
      }
    } else if (0x3F00..=0x3FFF).contains(&addr) {
      addr &= 0x001F;
      let idx: usize = match addr {
        0x0010 | 0x0014 | 0x0018 | 0x001C => usize::try_from(addr).unwrap() - 0x10,
        _ => addr.into()
      };
      self.palette_table[idx] = data;
    }
  }

  pub fn cpu_write(&mut self, address: u16, data: u8) {
    match address {
      0x00 => {
        self.ctrl_flags.0 = data;

        let ctrl_flags = self.ctrl_flags;
        self.tram_addr.set_nametable_x(ctrl_flags.nametable_x());
        self.tram_addr.set_nametable_y(ctrl_flags.nametable_y());
      }
      0x01 => {
        self.mask_flags.0 = data;
      },
      0x02 => {},
      0x03 => {},
      0x04 => {},
      0x05 => { // Scroll
        if self.address_latch {                    // Y
          self.tram_addr.set_fine_y(data & 0x07);
          self.tram_addr.set_coarse_y(data >> 3);
          self.address_latch = false;
        } else { // X
          self.fine_x = data & 0x07;
          self.tram_addr.set_coarse_x(data >> 3);
          self.address_latch = true;
        }
      },
      0x06 => { // PPU address
        if self.address_latch {
          self.tram_addr.set_lo_byte(data);
          let tram_addr = self.tram_addr;
          self.vram_addr = tram_addr;
          self.address_latch = false;
        } else {
          self.tram_addr.set_hi_byte(data);
          self.address_latch = true;
        }
      },
      0x07 => { // PPU data
        let mut vram_addr = self.vram_addr;
        let increment_val = if self.ctrl_flags.vram_addr_increment_mode() { 32 } else { 1 };
        self.vram_addr.0 = vram_addr.0.wrapping_add(increment_val);
        self.ppu_write(vram_addr.0, data);
      },
      _ => panic!("write_ppu_registers address: {} not in range", address),
    };
  }

  pub fn cpu_read(&mut self, address: u16, read_only: bool) -> u8 {
    if read_only {
      match address {
        0x00 => self.ctrl_flags.0,
        0x01 => self.mask_flags.0,
        0x02 => self.status_flags.0,
        0x03 => 0x00,
        0x04 => 0x00,
        0x05 => 0x00,
        0x06 => 0x00,
        0x07 => 0x00,
        _ => 0x00,
      }
    } else {
      match address {
        0x00 => 0x00,
        0x01 => 0x00,
        0x02 => {   // Status
          let status_flags = self.status_flags;
          let res = (status_flags.0 & 0xE0) | (self.ppu_data_buffer & 0x1F);
          self.status_flags.set_vertical_blank(false);
          self.address_latch = false;
          res
        }
        0x03 => 0x00,
        0x04 => 0x00,
        0x05 => 0x00,
        0x06 => 0x00,
        0x07 => {   // PPU data
          let mut data = self.ppu_data_buffer;
          let vram_addr = self.vram_addr;

          let increment_val = if self.ctrl_flags.vram_addr_increment_mode() { 32 } else { 1 };
          self.vram_addr.0 = vram_addr.0.wrapping_add(increment_val);

          if self.vram_addr.0 >= 0x3F00 {
            data = self.ppu_read(vram_addr.0).into();
          }
          data
        }
        _ => panic!("read_ppu_u8 address: {} not in range", address),
      }
    }
  }
}

#[test]
fn ppu_table_write_and_read() {
  let cart = Cartridge::mock_cartridge();
  let mut registers = Registers::new(Rc::new(RefCell::new(cart)));

  registers.ppu_write(0x2000u16, 1u8);
  let res = registers.ppu_read(0x2000u16);

  assert_eq!(res, 1u8)
}

#[test]
fn ppu_status_register_write_and_read() {
  let cart = Cartridge::mock_cartridge();
  let mut registers = Registers::new(Rc::new(RefCell::new(cart)));

  registers.status_flags.set_sprite_overflow(true);

  assert_eq!(registers.status_flags.sprite_overflow(), true);
  assert_eq!(registers.status_flags.0, 0b00_10_00_00);

  registers.status_flags.set_sprite_overflow(false);

  assert_eq!(registers.status_flags.sprite_overflow(), false);
  assert_eq!(registers.status_flags.0, 0b00_00_00_00);
}

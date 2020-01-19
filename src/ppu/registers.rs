use std::cell::{RefCell, RefMut};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::cartridge::Cartridge;
use crate::cartridge::rom_reading::Mirroring;

bitfield! {
  #[derive(Copy, Clone, PartialEq)]
  pub struct PpuCtrlFlags(u8); impl Debug;
    pub u8, nametable_x, _: 0;
    pub u8, nametable_y, _: 1;
    pub u8, vram_addr_increment_mode, _: 2;
    pub u8, pattern_sprite_table_addr, _: 3;
    pub u8, pattern_background_table_addr, _: 4;
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
//      print!("A ");
      self.get_mut_cartridge().rom.chr_rom[mapped_addr]
    } else {
      match addr {
        0x0000..=0x1FFF => {
//          print!("B ");
          let first_idx = usize::try_from((addr & 0x1000) >> 12).unwrap();
          let second_idx = usize::try_from(addr & 0x0FFF).unwrap();
          self.table_pattern[first_idx][second_idx]
        }
        0x2000..=0x3EFF => {
//          print!("C ");
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
        0x3F00..=0x3FFF => {
          addr &= 0x001F;
          let idx: usize = match addr {
            0x0010 | 0x0014 | 0x0018 | 0x001C => usize::try_from(addr).unwrap() - 0x10,
            _ => addr.into()
          };
//          print!("D {},{},{} ", addr, self.palette_table[usize::try_from(addr).unwrap()], (if self.mask_flags.grayscale() { 0x30 } else { 0x3F }));
          (self.palette_table[usize::try_from(idx).unwrap()] & (if self.mask_flags.grayscale() { 0x30 } else { 0x3F }))
        }
        _ => panic!("Address {} not in range", addr)
      }
    };
//    println!("PPU ORIG:{} ADDR:{} -> {}", address, addr, res);
    res
  }

  pub fn ppu_write(&mut self, address: u16, data: u8) {
    let mut addr = address & 0x3FFF;

    let (is_address_in_range, mapped_addr) = self.get_mut_cartridge().mapper.mapped_write_ppu_u8(addr);
    if is_address_in_range {
      self.get_mut_cartridge().rom.chr_rom[mapped_addr] = data;
    } else {
      match addr {
        0x0000..=0x1FFF => {
          let first_idx = usize::try_from((addr & 0x1000) >> 12).unwrap();
          let second_idx = usize::try_from(addr & 0x0FFF).unwrap();
          self.table_pattern[first_idx][second_idx] = data;
        }
        0x2000..=0x3EFF => {
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
        }
        0x3F00..=0x3FFF => {
          addr &= 0x001F;
          let idx: usize = match addr {
            0x0010 | 0x0014 | 0x0018 | 0x001C => usize::try_from(addr).unwrap() - 0x10,
            _ => addr.into()
          };
//          println!("Dwrite:{}", data);
          self.palette_table[idx] = data;
        }
        _ => panic!("Address {} not in range", addr)
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

use std::cell::{Ref, RefCell, RefMut};
use std::convert::{TryFrom, TryInto};
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
    pub u8, pattern_background, _: 4;
    pub u8, sprite_size, _: 5;
    pub u8, slave_mode, _: 6;
    pub u8, enable_nmi, _: 7;
}

impl PpuCtrlFlags {
  pub fn get_pattern_background(self) -> u16 {
    u16::try_from(self.pattern_background()).unwrap() * 0x1000
  }

  pub fn get_sprite_size(self) -> u16 {
    if self.sprite_size() {
      16
    } else {
      8
    }
  }

  pub fn get_sprite_tile_base(self) -> u16 {
    u16::try_from(self.pattern_sprite_table_addr()).unwrap() * 0x1000
  }
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

impl PpuMaskFlags {
  pub fn is_rendering(self) -> bool {
    self.show_sprites() || self.show_background()
  }

  pub fn is_rendering_background(self, x: usize) -> bool {
    self.show_background() && (self.show_background_in_left_margin() || x > 7)
  }

  pub fn is_rendering_sprites(self, x: usize) -> bool {
    self.show_sprites() && (self.show_sprites_in_left_margin() || x > 7)
  }
}

bitfield! {
  #[derive(Copy, Clone, PartialEq)]
  pub struct PpuStatusFlags(u8); impl Debug;
    pub u8, sprite_overflow, set_sprite_overflow:               5;
    pub u8, sprite_zero_hit, set_sprite_zero_hit:               6;
    pub u8, vertical_blank,  set_vertical_blank:                7;
}

bitfield! {
  #[derive(Copy, Clone, PartialEq)]
  pub struct AddressRegister(u16); impl Debug;
    pub u8,    coarse_x,     set_coarse_x:      4,  0;
    pub u8,    coarse_y,     set_coarse_y:      9,  5;
    pub u8,    nametable_x,  set_nametable_x:   10;
    pub u8,    nametable_y,  set_nametable_y:   11;
    pub u16,   address,      _:                 13, 0;
    pub u8,    fine_y,       set_fine_y:        14, 12;
    pub u8,    hi_byte,      set_hi_byte:       13, 8;
    pub u8,    lo_byte,      set_lo_byte:       7,  0;
}

pub fn get_nth_bit<T: Into<u16>, U: Into<u16>>(number: T, nth: U) -> u8 {
  ((number.into() >> nth.into()) & 1).try_into().unwrap()
}

#[derive(Clone)]
pub struct Registers {
  pub ctrl_flags: PpuCtrlFlags,
  pub mask_flags: PpuMaskFlags,
  pub status_flags: PpuStatusFlags,
  pub vram_addr: AddressRegister,
  pub tram_addr: AddressRegister,
  pub palette_table: [u8; 0x20],
  name_table: [[u8; 0x0400]; 2],
  address_latch: bool,
  pub ppu_data_buffer: u8,
  pub fine_x: u8,
  cartridge: Rc<RefCell<Box<Cartridge>>>,

  pub oam_address: u8,
  pub oam_ram: [u8; 0x100],
  sprite_count: u8,
  sprite_shifter_pattern_lo: u8,
  sprite_shifter_pattern_hi: u8,

  pub vblank_suppress: bool,
  pub force_nmi: bool,
  read_buffer: u8,
}

impl Registers {
  pub fn new(cartridge: Rc<RefCell<Box<Cartridge>>>) -> Registers {
    Registers {
      ctrl_flags: PpuCtrlFlags(0x00),
      mask_flags: PpuMaskFlags(0x00),
      status_flags: PpuStatusFlags(0x00),
      vram_addr: AddressRegister(0x00),
      tram_addr: AddressRegister(0x00),
      palette_table: [0; 0x20],
      name_table: [[0xFF; 0x0400]; 2],
      address_latch: false,
      ppu_data_buffer: 0x00,
      fine_x: 0x00,
      cartridge,

      oam_address: 0,
      oam_ram: [0u8; 0x100],
      sprite_count: 0,
      sprite_shifter_pattern_lo: 0,
      sprite_shifter_pattern_hi: 0,
      vblank_suppress: false,
      force_nmi: false,
      read_buffer: 0,
    }
  }

  pub fn reset(&mut self) {
    self.status_flags = PpuStatusFlags(0);
    self.mask_flags = PpuMaskFlags(0);
    self.ctrl_flags = PpuCtrlFlags(0);
    self.vram_addr = AddressRegister(0);
    self.tram_addr = AddressRegister(0);
    self.ppu_data_buffer = 0;
    self.fine_x = 0;
    self.oam_ram = [0; 0x0100];
    self.palette_table = [0; 0x20];
    self.name_table = [[0u8; 0x0400]; 2];
  }

  fn write_oam_address(&mut self, address: u8) {
    self.oam_address = address;
  }

  pub fn write_oam_data(&mut self, data: u8) {
    let idx = usize::try_from(self.oam_address).unwrap();
    self.oam_ram[idx] = data;
    self.oam_address = self.oam_address.wrapping_add(1);
  }

  fn read_oam_data(&self) -> u8 {
    let idx = usize::try_from(self.oam_address).unwrap();
    if idx % 4 == 2 {
      self.oam_ram[idx] & 0xE3
    } else {
      self.oam_ram[idx]
    }
  }

  pub fn get_mut_cartridge(&mut self) -> RefMut<Box<Cartridge>> {
    self.cartridge.borrow_mut()
  }

  fn get_cartridge(&self) -> Ref<Box<Cartridge>> {
    self.cartridge.borrow()
  }

  pub fn ppu_read_reg(&self, address: u16) -> u8 {
    let addr = address & 0x3FFF;
    if (0x0000..=0x1FFF).contains(&addr) {
      self.get_cartridge().mapper.mapped_read_ppu_u8(addr & 0x3FFF)
    } else if (0x2000..=0x3EFF).contains(&addr) {
      let (fst_idx, snd_idx) = mirror_name_table(self.get_cartridge().get_mirror_mode(), addr);
      self.name_table[fst_idx][snd_idx]
    } else if (0x3F00..=0x3FFF).contains(&addr) {
      self.palette_table[palette_table(addr)]
    } else {
      0
    }
  }

  pub fn ppu_write_reg(&mut self, address: u16, data: u8) {
    let addr = address & 0x3FFF;
    if (0x0000..=0x1FFF).contains(&addr) {
      self.get_mut_cartridge().mapper.mapped_write_ppu_u8(addr & 0x3FFF, data)
    } else if (0x2000..=0x3EFF).contains(&addr) {
      let (fst_idx, snd_idx) = mirror_name_table(self.get_cartridge().get_mirror_mode(), addr);
      self.name_table[fst_idx][snd_idx] = data;
    } else if (0x3F00..=0x3FFF).contains(&addr) {
      self.palette_table[palette_table(addr)] = data;
    }
  }

  pub fn bus_write_ppu_reg(&mut self, address: u16, data: u8) {
    self.ppu_data_buffer = data;
    match address % 8 {
      0x00 => self.write_control(data),
      0x01 => { self.mask_flags.0 = data; }
      0x02 => (),
      0x03 => self.write_oam_address(data),
      0x04 => self.write_oam_data(data),
      0x05 => self.write_scroll(data),
      0x06 => self.write_address(data),
      0x07 => self.write_data(data),
      _ => panic!("cpu_write_reg address: {} not in range", address),
    };
  }

  fn write_control(&mut self, data: u8) {
    if !self.ctrl_flags.enable_nmi() && PpuCtrlFlags(data).enable_nmi() {
      self.force_nmi = true;
    }
    self.ctrl_flags.0 = data;

    let ctrl_flags = self.ctrl_flags;
    self.tram_addr.set_nametable_x(ctrl_flags.nametable_x());
    self.tram_addr.set_nametable_y(ctrl_flags.nametable_y());
  }

  fn write_scroll(&mut self, data: u8) {
    if self.address_latch {
      self.tram_addr.set_fine_y(data);
      self.tram_addr.set_coarse_y(data >> 3);
      self.address_latch = false;
    } else {
      self.fine_x = data & 0x07;
      self.tram_addr.set_coarse_x(data >> 3);
      self.address_latch = true;
    }
  }

  fn write_address(&mut self, data: u8) {
    if self.address_latch {
      self.tram_addr.set_lo_byte(data);
      let tram_addr = self.tram_addr;
      self.vram_addr = tram_addr;
      self.address_latch = false;
    } else {
      self.tram_addr.set_hi_byte(data);
      self.address_latch = true;
    }
  }

  fn write_data(&mut self, data: u8) {
    let increment_val = if self.ctrl_flags.vram_addr_increment_mode() { 32 } else { 1 };
    self.ppu_write_reg(self.vram_addr.0, data);
    let addr = self.vram_addr.0;
    self.vram_addr = AddressRegister(addr.wrapping_add(increment_val));
  }

  pub fn bus_read_ppu_reg(&mut self, address: u16) -> u8 {
    let res = match address % 8 {
      0x00 => self.ppu_data_buffer,
      0x01 => self.ppu_data_buffer,
      0x02 => self.read_reg_status(),
      0x03 => self.ppu_data_buffer,
      0x04 => self.read_oam_data(),
      0x05 => self.ppu_data_buffer,
      0x06 => self.ppu_data_buffer,
      0x07 => self.read_ppu_data(),
      _ => panic!("cpu_read_reg address: {} not in range", address),
    };
    self.ppu_data_buffer = res;
    res
  }

  fn read_reg_status(&mut self) -> u8 {
    let res = self.status_flags.0;
    self.status_flags.set_vertical_blank(false);
    self.address_latch = false;
    self.vblank_suppress = true;
    res | (self.ppu_data_buffer & 0x1F)
  }

  fn read_ppu_data(&mut self) -> u8 {
    let vram_addr = self.vram_addr.address();
    if (0x3F00..=0x3FFF).contains(&vram_addr) {
      self.read_data() | (self.ppu_data_buffer & 0xC0)
    } else {
      self.read_data()
    }
  }

  fn read_data(&mut self) -> u8 {
    let addr = self.vram_addr.address();
    let increment_val = if self.ctrl_flags.vram_addr_increment_mode() { 32 } else { 1 };
    self.vram_addr.0 = self.vram_addr.0.wrapping_add(increment_val);
    self.buffered_read_byte(addr)
  }

  fn buffered_read_byte(&mut self, addr: u16) -> u8 {
    let prev_read_buffer = self.read_buffer;
    self.read_buffer = self.ppu_read_reg(addr);
    prev_read_buffer
  }
}

fn mirror_name_table(mirror_mode: Mirroring, addr: u16) -> (usize, usize) {
  let addr_range = addr & 0x0FFF;
  let idx = usize::try_from(addr_range & 0x03FF).unwrap();
  match mirror_mode {
    Mirroring::Vertical => {
      match addr_range {
        0x0000..=0x03FF => (0, idx),
        0x0400..=0x07FF => (1, idx),
        0x0800..=0x0BFF => (0, idx),
        0x0C00..=0x0FFF => (1, idx),
        _ => panic!("Unknown vertical mode table address")
      }
    }
    Mirroring::Horizontal => {
      match addr_range {
        0x0000..=0x03FF => (0, idx),
        0x0400..=0x07FF => (0, idx),
        0x0800..=0x0BFF => (1, idx),
        0x0C00..=0x0FFF => (1, idx),
        _ => panic!("Unknown horizontal mode table address")
      }
    }
  }
}

fn palette_table(addr: u16) -> usize {
  let address = (addr as usize) & 0x001F;
  match address {
    0x0010 | 0x0014 | 0x0018 | 0x001C => address - 0x10,
    _ => address
  }
}

#[cfg(test)]
mod test {
  use std::cell::RefCell;
  use std::rc::Rc;

  use crate::cartridge::Cartridge;
  use crate::ppu::registers::Registers;

  #[test]
  fn ppu_table_write_and_read() {
    let cart = Cartridge::mock_cartridge();
    let mut registers = Registers::new(Rc::new(RefCell::new(Box::new(cart))));

    registers.ppu_write_reg(0x2000u16, 1u8);
    let res = registers.ppu_read_reg(0x2000u16);

    assert_eq!(res, 1u8)
  }

  #[test]
  fn ppu_status_register_write_and_read() {
    let cart = Cartridge::mock_cartridge();
    let mut registers = Registers::new(Rc::new(RefCell::new(Box::new(cart))));

    registers.status_flags.set_sprite_overflow(true);

    assert_eq!(registers.status_flags.sprite_overflow(), true);
    assert_eq!(registers.status_flags.0, 0b00_10_00_00);

    registers.status_flags.set_sprite_overflow(false);

    assert_eq!(registers.status_flags.sprite_overflow(), false);
    assert_eq!(registers.status_flags.0, 0b00_00_00_00);
  }

  #[test]
  fn control_reg_write() {
    let cart = Cartridge::mock_cartridge();
    let mut registers = Registers::new(Rc::new(RefCell::new(Box::new(cart))));

    registers.bus_write_ppu_reg(0x2000, 0xAF);
    assert_eq!(registers.ctrl_flags.0, 0xAF)
  }

  #[test]
  fn mask_reg_write() {
    let cart = Cartridge::mock_cartridge();
    let mut registers = Registers::new(Rc::new(RefCell::new(Box::new(cart))));

    registers.bus_write_ppu_reg(0x2001, 0xBF);
    assert_eq!(registers.mask_flags.0, 0xBF)
  }

  #[test]
  fn oam_data_reg_write() {
    let cart = Cartridge::mock_cartridge();
    let mut registers = Registers::new(Rc::new(RefCell::new(Box::new(cart))));

    registers.bus_write_ppu_reg(0x2003, 0xF0);
    registers.bus_write_ppu_reg(0x2004, 0x04);
    assert_eq!(registers.oam_ram[0xF0], 0x04);
    assert_eq!(registers.oam_address, 0xF1);
  }

  #[test]
  fn scroll_reg_write() {
    let cart = Cartridge::mock_cartridge();
    let mut registers = Registers::new(Rc::new(RefCell::new(Box::new(cart))));

    registers.bus_write_ppu_reg(0x2005, 0xEE);
    assert_eq!(registers.fine_x, 6);
    assert_eq!(registers.tram_addr.coarse_x(), 0x1D);
    assert_eq!(registers.address_latch, true);


    registers.bus_write_ppu_reg(0x2005, 0xFA);
    assert_eq!(registers.fine_x, 6);
    assert_eq!(registers.tram_addr.coarse_y(), 0x1F);
    assert_eq!(registers.address_latch, false);
  }

  #[test]
  fn address_reg_write() {
    let cart = Cartridge::mock_cartridge();
    let mut registers = Registers::new(Rc::new(RefCell::new(Box::new(cart))));

    registers.bus_write_ppu_reg(0x2006, 0x1A);
    assert_eq!(registers.tram_addr.0, 0x1A00);
    assert_eq!(registers.address_latch, true);


    registers.bus_write_ppu_reg(0x2006, 0xB2);
    assert_eq!(registers.tram_addr.0, 0x1AB2);
    assert_eq!(registers.address_latch, false);
  }

  #[test]
  fn data_write() {
    let cart = Cartridge::mock_cartridge();
    let mut registers = Registers::new(Rc::new(RefCell::new(Box::new(cart))));

    registers.vram_addr.0 = 0x2000;
    registers.bus_write_ppu_reg(0x2007, 0xF0);
    assert_eq!(registers.ppu_read_reg(0x2000), 0xF0);
    assert_eq!(registers.vram_addr.0, 0x2001);

    registers.ctrl_flags.0 = 0x04;
    registers.bus_write_ppu_reg(0x2007, 0x0F);
    assert_eq!(registers.ppu_read_reg(0x2001), 0x0F);
    assert_eq!(registers.vram_addr.0, 0x2021);
  }

  #[test]
  fn read_ghost_bits_reg_write() {
    let cart = Cartridge::mock_cartridge();
    let mut registers = Registers::new(Rc::new(RefCell::new(Box::new(cart))));

    registers.bus_write_ppu_reg(0x2002, 0xFA);
    registers.status_flags.0 = 0;
    assert_eq!(registers.bus_read_ppu_reg(0x2002), 0x1A);
    assert_eq!(registers.bus_read_ppu_reg(0x2000), 0x1A);
    assert_eq!(registers.bus_read_ppu_reg(0x2001), 0x1A);
    assert_eq!(registers.bus_read_ppu_reg(0x2003), 0x1A);
    assert_eq!(registers.bus_read_ppu_reg(0x2005), 0x1A);
    assert_eq!(registers.bus_read_ppu_reg(0x2006), 0x1A);
  }

  #[test]
  fn delayed_read_data() {
    let cart = Cartridge::mock_cartridge();
    let mut registers = Registers::new(Rc::new(RefCell::new(Box::new(cart))));

    registers.ppu_write_reg(0x2001, 0x07);
    registers.ppu_write_reg(0x2002, 0x0B);
    registers.ppu_write_reg(0x2003, 0x0E);
    registers.vram_addr.0 = 0x2001;
    registers.bus_read_ppu_reg(0x2007);
    assert_eq!(registers.bus_read_ppu_reg(0x2007), 0x07);
    assert_eq!(registers.bus_read_ppu_reg(0x2007), 0x0B);
    assert_eq!(registers.bus_read_ppu_reg(0x2007), 0x0E);
  }
}

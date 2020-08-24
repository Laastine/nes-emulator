use std::cell::{Ref, RefCell, RefMut};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::cartridge::rom_reading::Rom;
use crate::mapper::Mapper;

#[derive(Clone)]
pub struct Mapper0 {
  prg_bank: usize,
  chr_bank: usize,
  rom: Rc<RefCell<Rom>>,
}

impl Mapper0 {
  pub fn new(rom: Rc<RefCell<Rom>>) -> Mapper0 {
    let rom_header = rom.borrow().rom_header;
    let prg_bank = rom_header.prg_rom_len / 0x4000;
    let chr_bank = rom_header.chr_rom_len / 0x2000;

    Mapper0 {
      prg_bank,
      chr_bank,
      rom,
    }
  }

  fn get_rom(&self) -> Ref<Rom> {
    self.rom.borrow()
  }

  fn get_mut_rom(&self) -> RefMut<Rom> {
    self.rom.borrow_mut()
  }
}

impl Mapper for Mapper0 {
  fn mapped_read_cpu_u8(&self, address: u16) -> u16 {
    let mask = if self.prg_bank > 1 { 0x7FFF } else { 0x3FFF };
    u16::try_from(self.get_rom().prg_rom[usize::try_from(address & mask).unwrap()]).unwrap()
  }

  fn mapped_write_cpu_u8(&mut self, address: u16, data: u8) {
    let mask = if self.prg_bank > 1 { 0x7FFF } else { 0x3FFF };
    self.get_mut_rom().prg_rom[usize::try_from(address & mask).unwrap()] = data;
  }

  fn mapped_read_ppu_u8(&self, address: u16) -> u8 {
    self.get_rom().chr_rom[usize::try_from(address).unwrap()]
  }

  fn mapped_write_ppu_u8(&mut self, address: u16, data: u8) {
    self.get_mut_rom().chr_rom[usize::try_from(address).unwrap()] = data;
  }
}

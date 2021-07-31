use std::cell::{Ref, RefCell};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::cartridge::rom_reading::Mirroring;
use crate::cartridge::rom_with_pager::RomData;
use crate::mapper::Mapper;
use crate::mapper::pager::Page;
use crate::mapper::pager::PageSize::{Eight, Sixteen};

#[derive(Clone)]
pub(crate) struct Mapper3 {
  chr_bank_select: usize,
  rom: Rc<RefCell<RomData>>,
  mirroring: Mirroring,
}

impl Mapper3 {
  pub fn new(rom: Rc<RefCell<RomData>>) -> Mapper3 {
    let mirroring = rom.borrow().rom_header.mirroring;

    Mapper3 {
      chr_bank_select: 0,
      rom,
      mirroring,
    }
  }

  fn get_rom(&self) -> Ref<RomData> {
    self.rom.borrow()
  }
}

impl Mapper for Mapper3 {
  fn mapped_read_cpu_u8(&self, address: u16) -> u8 {
    match address {
      0x8000..=0xBFFF => self.get_rom().prg_rom.read(Page::First(Sixteen), address - 0x8000),
      0xC000..=0xFFFF => self.get_rom().prg_rom.read(Page::Last(Sixteen), address - 0xC000),
      _ => panic!("Invalid mapped_read_cpu_u8 address 0x{:04X}", address),
    }
  }

  fn mapped_write_cpu_u8(&mut self, address: u16, data: u8) {
    if (0x8000..=0xFFFF).contains(&address) {
      self.chr_bank_select = usize::try_from(data & 0x03).unwrap()
    }
  }

  fn mapped_read_ppu_u8(&self, address: u16) -> u8 {
    self.get_rom().chr_rom.read(Page::FromNth(self.chr_bank_select, Eight), address)
  }

  fn mapped_write_ppu_u8(&mut self, _address: u16, _data: u8) {}

  fn mirroring(&self) -> Mirroring {
    self.mirroring
  }
}

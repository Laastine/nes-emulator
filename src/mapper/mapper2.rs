use std::cell::{Ref, RefCell, RefMut};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::cartridge::rom_with_pager::RomData;
use crate::mapper::Mapper;
use crate::mapper::pager::Page;
use crate::mapper::pager::PageSize::{Eight, Sixteen};
use crate::cartridge::CHR_ROM_BANK_SIZE;
use crate::cartridge::rom_reading::Mirroring;

#[derive(Clone)]
pub(crate) struct Mapper2 {
  prg_bank_select: usize,
  chr_rom_pages: usize,
  rom: Rc<RefCell<RomData>>,
  mirroring: Mirroring,
}

impl Mapper2 {
  pub fn new(rom: Rc<RefCell<RomData>>) -> Mapper2 {
    let mirroring = rom.borrow().rom_header.mirroring;
    let chr_rom_pages = rom.borrow().rom_header.chr_rom_len / CHR_ROM_BANK_SIZE;

    Mapper2 {
      prg_bank_select: 0,
      chr_rom_pages,
      rom,
      mirroring,
    }
  }

  fn get_rom(&self) -> Ref<RomData> {
    self.rom.borrow()
  }

  fn get_mut_rom(&self) -> RefMut<RomData> {
    self.rom.borrow_mut()
  }
}

impl Mapper for Mapper2 {
  fn mapped_read_cpu_u8(&self, address: u16) -> u8 {
    match address {
      0x6000..=0x7FFF => 0,
      0x8000..=0xBFFF => self.get_rom().prg_rom.read(Page::FromNth(self.prg_bank_select, Sixteen), address - 0x8000),
      0xC000..=0xFFFF => self.get_rom().prg_rom.read(Page::Last(Sixteen), address - 0xC000),
      _ => panic!("Invalid mapped_read_cpu_u8 address 0x{:04X}", address),
    }
  }

  fn mapped_write_cpu_u8(&mut self, address: u16, data: u8) {
    if (0x8000..=0xFFFF).contains(&address) {
      self.prg_bank_select = usize::try_from(data & 0x0F).unwrap()
    } else {
      panic!("Invalid mapped_write_cpu_u8 address 0x{:04X}", address)
    };
  }

  fn mapped_read_ppu_u8(&self, address: u16) -> u8 {
    if self.chr_rom_pages == 0 {
      self.get_rom().chr_ram.read(Page::First(Eight), address)
    } else {
      self.get_rom().chr_rom.read(Page::First(Eight), address)
    }
  }

  fn mapped_write_ppu_u8(&mut self, address: u16, data: u8) {
    if self.chr_rom_pages == 0 {
      self.get_mut_rom().chr_ram.write(Page::First(Eight), address, data);
    }
  }

  fn mirroring(&self) -> Mirroring {
    self.mirroring
  }
}

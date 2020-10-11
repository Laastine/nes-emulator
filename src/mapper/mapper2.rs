use std::cell::{Ref, RefCell, RefMut};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::cartridge::rom_reading::Rom;
use crate::mapper::Mapper;
use crate::cartridge::rom_with_pager::RomData;

const CHR_ROM_PAGE_SIZE: usize = 0x2000;
const PRG_ROM_PAGE_SIZE: usize = 0x4000;

#[derive(Clone)]
pub struct Mapper2 {
  prg_bank_select_lo: usize,
  prg_bank_select_hi: usize,
  chr_rom_pages: usize,
  rom: Rc<RefCell<RomData>>,
}

impl Mapper2 {
  pub fn new(rom: Rc<RefCell<RomData>>) -> Mapper2 {
    let rom_header = rom.borrow().rom_header;
    let chr_rom_pages = rom_header.chr_rom_len / CHR_ROM_PAGE_SIZE;
    dbg!(chr_rom_pages);
    Mapper2 {
      prg_bank_select_lo: 0,
      prg_bank_select_hi: 0,
      chr_rom_pages,
      rom,
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
  fn mapped_read_cpu_u8(&self, address: u16) -> u16 {
    let masked_addr = if (0x8000..=0xBFFF).contains(&address) {
      usize::try_from(address - 0x8000).unwrap()
    } else if (0xC000..=0xFFFF).contains(&address) {
      usize::try_from(address- 0xC000).unwrap()
    } else {
      panic!("Invalid mapped_read_cpu_u8 address {}", address)
    };
    u16::try_from(self.get_rom().prg_rom[usize::try_from(masked_addr).unwrap()]).unwrap()
  }

  fn mapped_write_cpu_u8(&mut self, address: u16, data: u8) {
    if (0x8000..=0xFFFF).contains(&address) {
      self.prg_bank_select_lo = usize::try_from(data & 0x0F).unwrap()
    } else {
      panic!("Invalid mapped_write_cpu_u8 address {}", address)
    };
  }

  fn mapped_read_ppu_u8(&self, address: u16) -> u8 {
    if self.chr_rom_pages == 0 {
      self.get_rom().chr_ram[usize::try_from(address).unwrap()]
    } else {
      self.get_rom().chr_rom[usize::try_from(address).unwrap()]
    }
  }

  fn mapped_write_ppu_u8(&mut self, address: u16, data: u8) {
    if self.chr_rom_pages == 0 {
      self.get_mut_rom().chr_ram[usize::try_from(address).unwrap()] = data;
    }
  }
}

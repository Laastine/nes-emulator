use std::convert::TryFrom;

use crate::cartridge::Cartridge;

/// NROM mapper
pub struct Mapper {
  cartridge: Cartridge,
  ram: [u8; 0x2000],
  chr_ram: Vec<u8>,
}

impl Mapper {
  pub fn new(rom_file: &str) -> Mapper {
    let ram = [0u8; 0x2000];
    let chr_ram = vec![];
    let cartridge = Cartridge::new(rom_file);
    let mapper = cartridge.rom.rom_header.mapper;
    if mapper != 0 {
      unimplemented!("Mapper {} not implemented", mapper)
    }
    Mapper {
      cartridge,
      ram,
      chr_ram,
    }
  }

  pub fn write_u8(&mut self, address: u16, data: u8) {
    let prg_rom = &self.cartridge.rom.prg_rom;
    let val = match address {
      0x6000..=0x7FFF => {
        let idx = usize::try_from(address - 0x6000).unwrap() % self.ram.len();
        self.ram[idx] = data;
      }
      0x8000..=0xFFFF => (),
      _ => panic!("Memory read from {}", address)
    };
  }

  pub fn read_u8(&self, address: u16) -> u16 {
    let prg_rom = &self.cartridge.rom.prg_rom;
    let val = match address {
      0x6000..=0x7FFF => {
        let idx = usize::try_from(address - 0x6000).unwrap() % self.ram.len();
        self.ram[idx]
      }
      0x8000..=0xFFFF => {
        let idx = usize::try_from(address - 0x6000).unwrap() % self.ram.len();
        prg_rom[idx]
      }
      _ => panic!("Memory read from {}", address)
    };
    u16::try_from(val).unwrap()
  }

  pub fn read_ppu_u8(&self, address: u16) -> u16 {
    unimplemented!();
  }

  pub fn write_ppu_u8(&self, address: u16, data: u8) -> u16 {
    unimplemented!();
  }

}

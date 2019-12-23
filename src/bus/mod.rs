use std::convert::{TryFrom, TryInto};

use crate::mapper::Mapper;
use crate::ppu::Ppu;
use crate::cartridge::Cartridge;

pub const MEM_SIZE: usize = 0x0800;

pub struct Bus {
  cartridge: Cartridge,
  pub ram: [u8; MEM_SIZE],
  mapper: Mapper,
}

impl Bus {
  pub fn new(rom_file: &str) -> Bus {
    let ram = [0u8; MEM_SIZE];
    let mapper = Mapper::new();
    let cartridge = Cartridge::new(rom_file);

    Bus {
      cartridge,
      ram,
      mapper,
    }
  }

  pub fn write_u8(&mut self, address: u16, data: u8) {
    match address {
      0x0000..=0x1FFF => {
        self.ram[usize::try_from(address & 0x07FF).unwrap()] = data;
      },
      0x2000..=0x3FFF => {
//        self.ppu.write_cpu_u8(address & 0x0007, data);
      }
      0x8000..=0xFFFF => {
        let mapped_addr = self.mapper.write_cpu_u8(address);
        self.cartridge.rom.prg_rom[usize::try_from(mapped_addr).unwrap()] = data;
      },
      _ => (),
    }
  }

  pub fn read_u8(&self, address: u16) -> u16 {
    match address {
      0x0000..=0x1FFF => {
        let idx = usize::try_from(address & 0x07FF).unwrap();
        u16::try_from(self.ram[idx]).unwrap()
      }
      0x2000..=0x3FFF => {
//        self.ppu.read_cpu_u8(address & 0x0007).into()
        0x00
      },
      0x8000..=0xFFFF => {
        let mapped_addr = self.mapper.read_cpu_u8(address);
        u16::try_from(self.cartridge.rom.prg_rom[usize::try_from(mapped_addr).unwrap()]).unwrap()
      },
      _ => 0,
    }
  }
}

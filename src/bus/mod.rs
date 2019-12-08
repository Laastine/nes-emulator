use std::convert::{TryFrom, TryInto};

use crate::mapper::Mapper;

pub const MEM_SIZE: usize = 0x0800;

pub struct Bus {
  pub ram: [u8; MEM_SIZE],
  mapper: Mapper,
}

impl Bus {
  pub fn new(rom_file: &str) -> Bus {
    let ram: [u8; MEM_SIZE] = [0u8; MEM_SIZE];
    let mapper = Mapper::new(rom_file);
    Bus {
      ram,
      mapper,
    }
  }

  pub fn write_u8(&mut self, address: u16, data: u8) {
    match address {
      0x0000..=0x07FF => {
        self.ram[usize::try_from(address).unwrap()] = data;
      }
      0x6000..=0xFFFF => {
        self.mapper.write_u8(address, data);
      }
      _ => {
        panic!("Cannot write to {}", address);
      }
    }
  }

  pub fn read_u8(&self, address: u16) -> u16 {
    match address {
      0x0000..=0x07FF => {
        u16::try_from(self.ram[usize::try_from(address).unwrap()]).unwrap()
      }
      0x6000..=0xFFFF => {
        self.mapper.read_u8(address)
      }
      _ => {
        panic!("Cannot read from {}", address);
      }
    }
  }
}

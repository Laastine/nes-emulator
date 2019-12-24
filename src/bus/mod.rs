use std::convert::TryFrom;

use crate::cartridge::Cartridge;
use crate::mapper::Mapper;

pub const MEM_SIZE: usize = 0x0800;

#[derive(Clone)]
pub struct Bus {
  cartridge: Cartridge,
  pub ram: [u8; MEM_SIZE],
  mapper: Mapper,
  system_cycles: u64
}

impl Bus {
  pub fn new(cartridge: Cartridge, mapper: Mapper) -> Bus {
    let ram = [0u8; MEM_SIZE];
    let system_cycles = 0;

    Bus {
      cartridge,
      mapper,
      ram,
      system_cycles,
    }
  }

  pub fn write_u8(&mut self, address: u16, data: u8) {
    match address {
      0x0000..=0x1FFF => {
        self.ram[usize::try_from(address & 0x07FF).unwrap()] = data;
      }
      0x2000..=0x3FFF => {
        self.write_cpu_u8(address & 0x0007, data);
      }
      0x8000..=0xFFFF => {
        let mapped_addr = usize::try_from(self.mapper.write_cpu_u8(address)).unwrap();
        {
          let prg_rom = self.cartridge.get_prg_rom();
          prg_rom[mapped_addr] = data
        };
      }
      _ => (),
    }
  }

  pub fn read_u8(&mut self, address: u16) -> u16 {
    match address {
      0x0000..=0x1FFF => {
        let idx = usize::try_from(address & 0x07FF).unwrap();
        u16::try_from(self.ram[idx]).unwrap()
      }
      0x2000..=0x3FFF => {
        self.read_cpu_u8(address & 0x0007).into()
      },
      0x8000..=0xFFFF => {
        let mapped_addr = usize::try_from(self.mapper.read_cpu_u8(address)).unwrap();
        u16::try_from({
          let prg_rom = self.cartridge.get_prg_rom();
          prg_rom[mapped_addr]
        }).unwrap()
      }
      _ => 0,
    }
  }

  fn write_cpu_u8(&self, address: u16, data: u8) {
    unimplemented!()
  }

  fn read_cpu_u8(&self, address: u16) -> u8 {
    match address {
      0x00 => 0x00,
      0x01 => 0x00,
      0x02 => 0x00,
      0x03 => 0x00,
      0x04 => 0x00,
      0x05 => 0x00,
      0x06 => 0x00,
      0x07 => 0x00,
      0x08 => 0x00,
      _ => 0x00,
    }
  }

  pub fn reset(&mut self) {
    self.system_cycles = 0;
  }

  fn clock(&mut self) {
//    self.ppu.clock();

    if self.system_cycles % 3 == 0 {
      unimplemented!("Cpu clock call here");
    }

    self.system_cycles += 1;
  }
}

use std::convert::{TryInto, TryFrom};

pub const MEM_SIZE: usize = 64 * 1024;

pub struct Bus {
  pub memory: [u8; MEM_SIZE],
}

impl Bus {
  pub fn new() -> Bus {
    let memory: [u8; MEM_SIZE] = [0u8; MEM_SIZE];
    Bus {
      memory,
    }
  }

  pub fn write_u8(&mut self, address: u16, data: u8) {
    if address <= (0xFFFF).try_into().unwrap() {
      self.memory[address as usize] = data;
    } else {
      panic!("Cannot write to {}", address);
    }
  }

  pub fn read_u8(&self, address: u16) -> u16 {
    if address <= (0xFFFF).try_into().unwrap() {
      u16::try_from(self.memory[address as usize]).unwrap()
    } else {
      panic!("Memory read from {}", address);
    }
  }
}

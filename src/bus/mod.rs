use std::convert::TryInto;

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
    if address <= (0xFFFF as u16).try_into().unwrap() {
      self.memory[address as usize] = data;
    } else {
      panic!("Cannot write to {}", address);
    }
  }

  pub fn read_u8(&self, address: u16) -> u16 {
    match address {
      0x0000..=0xFFFF => {
        let memory_offset = (address as usize) % self.memory.len();
        self.memory[memory_offset] as u16
      }
    }
  }
}

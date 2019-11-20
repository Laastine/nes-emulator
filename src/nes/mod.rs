use crate::cpu::Cpu;
use crate::rom::Rom;

const MEM_SIZE: usize = 64 * 1024;

pub enum FLAGS6502 {
  C = 1,    // Carry
  Z = 2,    // Zero
  I = 4,    // Disable Interrupts
  D = 8,    // Decimal Mode
  B = 16,   // Break
  U = 32,   // Unused
  V = 64,   // Overflow
  N = 128,  // Negative
}

pub struct Nes {
  cpu: Cpu,
  memory: [u8; MEM_SIZE],
  rom: Rom,
}

impl Nes {
  pub fn new(rom: Rom) -> Nes {
    let pc_bytes = &rom.rom[0x3FFC..=0x3FFD];
    let pc = (pc_bytes[0] as u16) | ((pc_bytes[1] as u16) << 8);

    let cpu = Cpu::new(pc);

    let memory: [u8; MEM_SIZE] = [0u8; MEM_SIZE];

    Nes {
      cpu,
      memory,
      rom
    }
  }

  pub fn write(&mut self, address: u16, data: u8) {
    if address >= 0 && address <= MEM_SIZE as u16 {
      self.memory[address as usize] = data;
    } else {
      panic!("Cannot write to {}", address);
    }
  }

  pub fn read_u8(&self, address: u16) -> u8 {
    if address >= 0 && address <= MEM_SIZE as u16 {
      self.memory[address as usize]
    } else {
      panic!("Cannot read from {}", address);
    }
  }
}

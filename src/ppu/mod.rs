use crate::bus::Bus;
use std::rc::Rc;
use std::cell::Cell;

pub struct Ppu {
  cycles: u32,
  scan_line: u32,
}

impl Ppu {
  pub fn new() -> Ppu {
    let cycles = 0;
    let scan_line = 0;
    Ppu {
      cycles,
      scan_line,
    }
  }

  pub fn write_cpu_u8(&self, address: u16, data: u8) {
    unimplemented!()
  }

  pub fn read_cpu_u8(&self, address: u16) -> u8 {
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

  pub fn clock(&mut self) {
    self.cycles += 1;
  }
}

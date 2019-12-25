use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use crate::bus::Bus;
use std::convert::TryFrom;

pub struct Ppu {
  bus: Rc<RefCell<Bus>>,
  cycles: u32,
  scan_line: u32,
  pub is_frame_ready: bool,
}

impl Ppu {
  pub fn new(bus: Rc<RefCell<Bus>>) -> Ppu {
    let cycles = 0;
    let scan_line = 0;
    Ppu {
      bus,
      cycles,
      scan_line,
      is_frame_ready: false,
    }
  }

  pub fn get_mut_bus(&mut self) -> RefMut<Bus> {
    self.bus.borrow_mut()
  }

  pub fn write_cpu_u8(&mut self, address: u16, data: u8) {
    let addr = match address {
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
    };
    {
      let mut bus = self.get_mut_bus();
      bus.write_u8(addr, data);
    }
  }

  pub fn read_cpu_u8(&mut self, address: u16) -> u8 {
    let addr = match address {
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
    };
    {
      let mut bus = self.get_mut_bus();
      u8::try_from(bus.read_u8(addr)).unwrap()
    }
  }

  pub fn clock(&mut self) {
    self.cycles += 1;

    if self.cycles > 340 {
      self.cycles = 0;
      self.scan_line += 1;
      if self.scan_line > 260 {
        self.scan_line -= 1;
        self.is_frame_ready = true;
      }
    }
  }
}

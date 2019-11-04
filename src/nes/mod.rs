use crate::bus::{Bus};
use crate::rom::Rom;
use crate::cpu::Cpu;

pub struct Nes {
  bus: Bus,
  rom: Rom,
}

impl Nes {
  pub fn new(rom: Rom) -> Nes {

    let mut bus = Bus::new();
    let cpu = Cpu::new(&mut bus);

    Nes {
      bus,
      rom,
    }
  }

  pub fn exec(&mut self) {
    unimplemented!()
  }
}

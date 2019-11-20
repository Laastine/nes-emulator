#[derive(Debug)]
pub struct Cpu {
  pc: u16,
  a: u8,
  x: u8,
  y: u8,
  s: u8,
  p: u8,
}

impl Cpu {
  pub fn new(pc: u16) -> Cpu {
    Cpu { pc, a: 0, x: 0, y: 0, s: 0xFD, p: 0x34 }
  }
}

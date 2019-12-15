pub struct Ppu {}

impl Ppu {
  pub fn new() -> Ppu {
    Ppu {}
  }

  pub fn write_cpu_u8(&self, address: u16, data: u8) {
    unimplemented!()
  }
  pub fn read_cpu_u8(&self, address: u16) -> u8 {
    match address {
      _ => 0x00
    }
  }
}

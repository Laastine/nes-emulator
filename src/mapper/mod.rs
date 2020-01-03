/// NROM mapper
#[derive(Copy, Clone)]
pub struct Mapper {}

impl Mapper {
  pub fn new() -> Mapper {
    Mapper {}
  }

  pub fn mapped_read_cpu_u8(self, address: u16) -> u16 {
    address & 0x3FFF
  }

  pub fn mapped_write_cpu_u8(self, address: u16) -> u16 {
    address & 0x3FFF
  }

  pub fn mapped_read_ppu_u8(self, address: u16) -> u16 {
    unimplemented!();
  }

  pub fn mapped_write_ppu_u8(self, address: u16) -> u16 {
    unimplemented!();
  }
}

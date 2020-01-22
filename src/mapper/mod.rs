use std::convert::TryFrom;

/// NROM mapper
#[derive(Copy, Clone)]
pub struct Mapper {
  prg_bank: usize,
  chr_bank: usize,
}

impl Mapper {
  pub fn new(prg_bank: usize, chr_bank: usize) -> Mapper {
    Mapper {
      prg_bank,
      chr_bank,
    }
  }

  pub fn mapped_read_cpu_u8(self, address: u16) -> (bool, usize) {
    let mask = if self.prg_bank > 1 { 0x7FFF } else { 0x3FFF };
    let mapped_addr = usize::try_from(address & mask).unwrap();
    let is_mappable = (0x8000..=0xFFFF).contains(&address);
    (is_mappable, mapped_addr)
  }

  pub fn mapped_read_ppu_u8(self, address: u16) -> (bool, usize) {
    let mapped_addr = usize::try_from(address).unwrap();
    let is_mappable = (0x0000..=0x1FFF).contains(&address);
    (is_mappable, mapped_addr)
  }

  pub fn mapped_write_ppu_u8(self, address: u16) -> (bool, usize) {
    let mapped_addr = usize::try_from(address).unwrap();
    let is_mappable = (0x0000..=0x1FFF).contains(&address) && self.chr_bank == 0;
    (is_mappable, mapped_addr)
  }
}

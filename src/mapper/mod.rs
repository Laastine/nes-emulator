use std::convert::TryFrom;

pub trait Mapper: MapperClone {
  fn mapped_read_cpu_u8(&self, address: u16) -> (bool, usize);
  fn mapped_read_ppu_u8(&self, address: u16) -> (bool, usize);
  fn mapped_write_ppu_u8(&self, address: u16) -> (bool, usize);
}

pub trait MapperClone {
  fn clone_mapper(&self) -> Box<dyn Mapper>;
}

impl<T> MapperClone for T
  where T: 'static + Mapper + Clone, {
  fn clone_mapper(&self) -> Box<dyn Mapper> {
      Box::new(self.clone())
  }
}

impl Clone for Box<dyn Mapper> {
  fn clone(&self) -> Box<dyn Mapper> {
    self.clone_mapper()
  }
}

/// NROM mapper
#[derive(Copy, Clone)]
pub struct Mapper0 {
  prg_bank: usize,
  chr_bank: usize,
}

impl Mapper0 {
  pub fn new(prg_bank: usize, chr_bank: usize) -> Mapper0 {
    Mapper0 {
      prg_bank,
      chr_bank,
    }
  }
}

impl Mapper for Mapper0 {
  fn mapped_read_cpu_u8(&self, address: u16) -> (bool, usize) {
    let mask = if self.prg_bank > 1 { 0x7FFF } else { 0x3FFF };
    let mapped_addr = usize::try_from(address & mask).unwrap();
    let is_mappable = (0x8000..=0xFFFF).contains(&address);
    (is_mappable, mapped_addr)
  }

  fn mapped_read_ppu_u8(&self, address: u16) -> (bool, usize) {
    let mapped_addr = usize::try_from(address).unwrap();
    let is_mappable = (0x0000..=0x1FFF).contains(&address);
    (is_mappable, mapped_addr)
  }

  fn mapped_write_ppu_u8(&self, address: u16) -> (bool, usize) {
    let mapped_addr = usize::try_from(address).unwrap();
    let is_mappable = (0x0000..=0x1FFF).contains(&address) && self.chr_bank == 0;
    (is_mappable, mapped_addr)
  }
}

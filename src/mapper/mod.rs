use crate::cartridge::rom_reading::Mirroring;

pub mod mapper0;
// pub mod mapper1;
pub mod mapper2;
pub mod mapper3;
pub mod mapper4;
pub mod pager;

pub trait Mapper: MapperClone {
  fn mapped_read_cpu_u8(&self, address: u16) -> u8;
  fn mapped_write_cpu_u8(&mut self, address: u16, data: u8);
  fn mapped_read_ppu_u8(&self, address: u16) -> u8;
  fn mapped_write_ppu_u8(&mut self, address: u16, data: u8);
  fn mirroring(&self) -> Mirroring;
  fn irq_flag(&self) -> bool {
    false
  }
  fn signal_scanline(&mut self) {}
  fn clear_irq_flag(&mut self) {}
}

pub trait MapperClone {
  fn clone_mapper(&self) -> Box<dyn Mapper>;
}

impl<T> MapperClone for T where T: 'static + Mapper + Clone, {
  fn clone_mapper(&self) -> Box<dyn Mapper> {
    Box::new(self.clone())
  }
}

impl Clone for Box<dyn Mapper> {
  fn clone(&self) -> Box<dyn Mapper> {
    self.clone_mapper()
  }
}

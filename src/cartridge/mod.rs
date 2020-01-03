use std::fs;

use crate::cartridge::rom_reading::{Rom, Mirroring};
use crate::mapper::Mapper;

pub mod rom_reading;

#[derive(Clone)]
pub struct Cartridge {
  pub rom: Rom,
  pub mapper: Mapper,
}

impl Cartridge {
  pub fn new(rom_file: &str) -> Cartridge {
    let mapper = Mapper::new();

    let rom_bytes = fs::read(rom_file).expect("Rom file read error");
    let rom = Rom::read_from_file(rom_bytes.into_iter());

    Cartridge { mapper, rom }
  }

  pub fn get_prg_rom(&mut self) -> &mut Vec<u8> {
    &mut self.rom.prg_rom
  }

  pub fn cart_cpu_read(&self, address: u16) -> u8 {
    unimplemented!()
  }

  pub fn cart_cpu_write(&mut self, address: u16, data: u8) {
    unimplemented!()
  }

  pub fn cart_ppu_read(&self, address: u16) -> u8 {
    unimplemented!()
  }

  pub fn cart_ppu_write(&mut self, address: u16, data: u8) {
    unimplemented!()
  }

  pub fn get_mirror_mode(&self) -> Mirroring {
    self.rom.rom_header.mirroring
  }
}

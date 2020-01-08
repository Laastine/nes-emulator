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
    let rom_bytes = fs::read(rom_file).expect("Rom file read error");
    let rom = Rom::read_from_file(rom_bytes.into_iter());

    let prg_banks = rom.rom_header.prg_rom_len / 0x4000;
    let chr_banks = rom.rom_header.chr_rom_len / 0x2000;

    let mapper = Mapper::new(prg_banks, chr_banks);

    Cartridge { mapper, rom }
  }

  pub fn mock_cartridge() -> Cartridge {
    let rom = Rom::mock_rom();
    let prg_banks = rom.rom_header.prg_rom_len / 0x4000;
    let chr_banks = rom.rom_header.chr_rom_len / 0x2000;

    let mapper = Mapper::new(prg_banks, chr_banks);

    Cartridge { mapper, rom }
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

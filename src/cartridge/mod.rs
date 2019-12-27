use std::fs;

use crate::cartridge::rom::Rom;

mod rom;

#[derive(Clone)]
pub struct Cartridge {
  pub rom: Rom,
}

impl Cartridge {
  pub fn new(rom_file: &str) -> Cartridge {
    let rom_bytes = fs::read(rom_file).expect("Rom file read error");
    let rom = Rom::read_from_file(rom_bytes.into_iter());

    Cartridge { rom }
  }

  pub fn get_prg_rom(&mut self) -> &mut Vec<u8> {
    &mut self.rom.prg_rom
  }
}

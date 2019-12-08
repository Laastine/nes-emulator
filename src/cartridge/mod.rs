use std::convert::{TryFrom, TryInto};
use std::fs;

use crate::cartridge::rom::Rom;

mod rom;

pub struct Cartridge {
  pub rom: Rom,
}

impl Cartridge {
  pub fn new(rom_file: &str) -> Cartridge {
    let rom_bytes = fs::read(rom_file).expect("Rom file read error");
    let rom = Rom::read_from_file(rom_bytes.into_iter());

    Cartridge {
      rom,
    }
  }
}

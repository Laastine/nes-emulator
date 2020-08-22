use crate::cartridge::rom_reading::{Mirroring, Rom};
use crate::mapper::Mapper;

pub mod rom_reading;

#[derive(Clone)]
pub struct Cartridge {
  pub rom: Rom,
  pub mapper: Mapper,
}

impl Cartridge {
  pub fn new(rom: Rom) -> Cartridge {
    let prg_banks = rom.rom_header.prg_rom_len / 0x4000;
    let chr_banks = rom.rom_header.chr_rom_len / 0x2000;

    let mapper = Mapper::new(prg_banks, chr_banks);

    Cartridge { mapper, rom }
  }

  pub fn get_mirror_mode(&self) -> Mirroring {
    self.rom.rom_header.mirroring
  }
}

#[cfg(test)]
mod test {
  use crate::cartridge::Cartridge;
  use crate::cartridge::rom_reading::{Mirroring, Rom};
  use crate::mapper::Mapper;

  impl Cartridge {
    pub fn mock_cartridge() -> Cartridge {
      let rom = Rom::mock_rom();
      let prg_banks = rom.rom_header.prg_rom_len / 0x4000;
      let chr_banks = rom.rom_header.chr_rom_len / 0x2000;

      let mapper = Mapper::new(prg_banks, chr_banks);

      Cartridge { mapper, rom }
    }
  }

  #[test]
  fn rom_read_test() {
    let cart = Cartridge::mock_cartridge();
    assert_eq!(cart.get_mirror_mode(), Mirroring::Horizontal);
  }
}

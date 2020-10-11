use std::cell::RefCell;
use std::rc::Rc;

use crate::cartridge::rom_reading::RomHeader;
use crate::cartridge::rom_with_pager::RomData;
use crate::mapper::{Mapper, mapper0::Mapper0, mapper2::Mapper2};

pub mod rom_reading;
pub mod rom_with_pager;

pub const CHR_ROM_BANK_SIZE: usize = 0x2000;
pub const PRG_ROM_BANK_SIZE: usize = 0x4000;

#[derive(Clone)]
pub struct Cartridge {
  pub mapper: Box<dyn Mapper>,
  pub rom_header: RomHeader,
}

impl Cartridge {
  pub fn new(rom: Rc<RefCell<RomData>>) -> Box<Cartridge> {
    let rom_header = rom.borrow().rom_header;

    let mapper: Box<dyn Mapper> = match rom_header.mapper {
      0 => Box::new(Mapper0::new(rom)),
      2 => Box::new(Mapper2::new(rom)),
      _ => panic!("Mapper {} not implemented", rom_header.mapper),
    };

    Box::from(Cartridge { mapper, rom_header })
  }
}

#[cfg(test)]
mod test {
  use crate::cartridge::{Cartridge, PRG_ROM_BANK_SIZE, CHR_ROM_BANK_SIZE};
  use crate::cartridge::rom_reading::{Mirroring, Rom};
  use crate::mapper::Mapper0;
  use crate::mapper::mapper0::Mapper0;

  impl Cartridge {
    pub fn mock_cartridge() -> Cartridge {
      let rom = Rom::mock_rom();
      let prg_banks = rom.rom_header.prg_rom_len / PRG_ROM_BANK_SIZE;
      let chr_banks = rom.rom_header.chr_rom_len / CHR_ROM_BANK_SIZE;

      let mapper = Box::new(Mapper0::new(rom));

      Cartridge { mapper }
    }
  }
}

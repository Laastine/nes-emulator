use std::cell::RefCell;
use std::rc::Rc;

use crate::cartridge::rom_reading::{Rom, RomHeader};
use crate::mapper::{Mapper, mapper0::Mapper0, mapper2::Mapper2, mapper3::Mapper3, mapper4::Mapper4};
use crate::cartridge::rom_with_pager::RomData;

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
  pub fn new(rom_bytes: Vec<u8>) -> Box<Cartridge> {
    let rom = Rom::read_from_file(rom_bytes.into_iter());
    let rom_header = rom.rom_header;

    let rom_ref = Rc::new(RefCell::new(RomData::new(rom)));

    let mapper: Box<dyn Mapper> = match rom_header.mapper {
      0 => Box::new(Mapper0::new(rom_ref)),
      2 => Box::new(Mapper2::new(rom_ref)),
      3 => Box::new(Mapper3::new(rom_ref)),
      4 => Box::new(Mapper4::new(rom_ref)),
      _ => panic!("Mapper {} not implemented", rom_header.mapper),
    };

    Box::from(Cartridge { mapper, rom_header })
  }

  pub fn irq_flag(&self) -> bool {
    self.mapper.irq_flag()
  }
}

#[cfg(test)]
mod test {
  use crate::cartridge::Cartridge;
  use crate::cartridge::rom_reading::Rom;
  use crate::mapper::mapper0::Mapper0;
  use std::cell::RefCell;
  use std::rc::Rc;
  use crate::cartridge::rom_with_pager::RomData;

  impl Cartridge {
    pub fn mock_cartridge() -> Cartridge {
      let rom = Rom::mock_rom();

      let rom_header = rom.rom_header;
      let rom_ref = Rc::new(RefCell::new(RomData::new(rom)));
      let mapper = Box::new(Mapper0::new(rom_ref));

      Cartridge { mapper, rom_header }
    }
  }
}

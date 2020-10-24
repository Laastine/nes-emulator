use crate::mapper::pager::Pager;
use crate::cartridge::rom_reading::{Rom, RomHeader};

pub struct RomData {
  pub rom_header: RomHeader,
  pub prg_rom: Pager,
  pub prg_ram: Pager,
  pub chr_rom: Pager,
  pub chr_ram: Pager,
}

impl RomData {
  pub fn new(rom: Rom) -> RomData {
    let rom_header = rom.rom_header;
    let prg_rom = Pager::new(rom.prg_rom.to_vec());
    let prg_ram = Pager::new(rom.prg_ram.to_vec());
    let chr_rom = Pager::new(rom.chr_rom.to_vec());
    let chr_ram = Pager::new(rom.chr_ram.to_vec());

    RomData {
      rom_header,
      prg_rom,
      prg_ram,
      chr_rom,
      chr_ram
    }
  }
}
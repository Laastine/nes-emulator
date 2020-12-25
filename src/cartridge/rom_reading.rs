use std::iter::Iterator;

#[derive(Copy, Clone, Debug)]
pub struct RomHeader {
  pub prg_rom_len: usize,
  pub chr_rom_len: usize,
  pub prg_ram_len: usize,
  pub chr_ram_len: usize,
  pub mirroring: Mirroring,
  pub mapper: u8,
  pub flag_persistent: bool,
  pub flag_trainer: bool,
  pub flag_vs_unisystem: bool,
  pub flag_playchoice_10: bool,
  pub flag_bus_conflicts: bool,
}

const PRG_ROM_PAGE_SIZE: usize = 0x4000;
const PRG_RAM_PAGE_SIZE: usize = 0x2000;
const CHR_ROM_PAGE_SIZE: usize = 0x2000;
const CHR_RAM_PAGE_SIZE: usize = 0x2000;

#[derive(Clone, Debug)]
pub(crate) struct Rom {
  pub rom_header: RomHeader,
  pub prg_rom: Vec<u8>,
  pub prg_ram: Vec<u8>,
  pub chr_rom: Vec<u8>,
  pub chr_ram: Vec<u8>,
}

impl Rom {
  pub fn read_from_file(mut rom_bytes: impl Iterator<Item=u8>) -> Rom {
    let mut bytes = &mut rom_bytes;

    let first_4_bytes = (&mut bytes).take(4);
    if b"NES\x1A".iter().cloned().ne(first_4_bytes) {
      panic!("Invalid ROM header");
    }

    let prg_rom_pagse = bytes.next().unwrap_or_else(|| panic!("prg_rom read error"));
    let chr_rom_pages = bytes.next().unwrap_or_else(|| panic!("chr_rom read error"));
    let flags_6 = bytes.next().unwrap_or_else(|| panic!("flags_6 read error"));
    let flags_7 = bytes.next().unwrap_or_else(|| panic!("flags_7 read error"));
    let flags_8 = bytes.next().unwrap_or_else(|| panic!("flags_8 read error"));
    let _flags_9 = bytes.next().unwrap_or_else(|| panic!("flags_9 read error"));
    let flags_10 = bytes.next().unwrap_or_else(|| panic!("flags_10 read error"));

    let zeros = (&mut bytes).take(5);
    if [0, 0, 0, 0, 0].iter().cloned().ne(zeros) {
      panic!("Non-zero bits found on unused block")
    }

    let flag_mirror = (flags_6 & 0x01) > 0x00;
    let flag_persistent = (flags_6 & 0x02) > 0x00;
    let flag_trainer = (flags_6 & 0x04) > 0x00;
    let flag_four_screen_vram = (flags_6 & 0x08) > 0x00;
    let mapper_lo = (flags_6 & 0xF0) >> 4;

    let flag_vs_unisystem = (flags_7 & 0x01) > 0x00;
    let flag_playchoice_10 = (flags_7 & 0x02) > 0x00;
    let flag_rom_format = (flags_7 & 0x0C) >> 2;
    let mapper_hi = flags_7 & 0xF0;

    let flag_bus_conflicts = (flags_10 & 0x20) > 0x00;

    if flag_rom_format == 2 {
      unimplemented!("NES 2.0 ROM format not implemented");
    }

    let prg_rom_len = prg_rom_pagse as usize * PRG_ROM_PAGE_SIZE;
    let chr_rom_len = chr_rom_pages as usize * CHR_ROM_PAGE_SIZE;

    let prg_ram_size = if flags_8 > 0 { flags_8 } else { 1 };
    let prg_ram_len = prg_ram_size as usize * PRG_RAM_PAGE_SIZE;

    let chr_ram_len = if chr_rom_pages == 0 { CHR_RAM_PAGE_SIZE } else { chr_rom_pages as usize * CHR_RAM_PAGE_SIZE };

    let mirroring = match (flag_mirror, flag_four_screen_vram) {
      (true, false) => Mirroring::Vertical,
      (false, false) => Mirroring::Horizontal,
      _ => panic!("Mirroring mode {}, {} not supported", flag_mirror, flag_four_screen_vram)
    };

    let mapper = mapper_lo | mapper_hi;

    let rom_header = RomHeader {
      prg_rom_len,
      chr_rom_len,
      prg_ram_len,
      chr_ram_len,
      mirroring,
      mapper,
      flag_persistent,
      flag_trainer,
      flag_vs_unisystem,
      flag_playchoice_10,
      flag_bus_conflicts,
    };

    let prg_rom = bytes.take(rom_header.prg_rom_len).collect::<Vec<u8>>();
    if prg_rom.len() != rom_header.prg_rom_len {
      panic!("Couldn't initialize PRG ROM");
    }

    let prg_ram = vec![0u8; rom_header.prg_ram_len];

    let chr_rom = bytes.take(rom_header.chr_rom_len).collect::<Vec<u8>>();
    if chr_rom.len() != rom_header.chr_rom_len {
      panic!("Couldn't initialize CHR ROM");
    }

    let chr_ram = vec![0u8; chr_ram_len];

    if bytes.next().is_some() {
      panic!("Unexpected ROM size");
    }

    Rom {
      rom_header,
      prg_rom,
      prg_ram,
      chr_rom,
      chr_ram,
    }
  }

  #[allow(dead_code)]
  pub fn mock_rom() -> Rom {
    let rom_header = RomHeader {
      prg_rom_len: PRG_ROM_PAGE_SIZE,
      chr_rom_len: CHR_ROM_PAGE_SIZE,
      prg_ram_len: PRG_RAM_PAGE_SIZE,
      chr_ram_len: CHR_RAM_PAGE_SIZE,
      mirroring: Mirroring::Horizontal,
      mapper: 0,
      flag_persistent: false,
      flag_trainer: false,
      flag_vs_unisystem: false,
      flag_playchoice_10: false,
      flag_bus_conflicts: false,
    };

    Rom {
      rom_header,
      prg_rom: vec![0u8; rom_header.prg_rom_len],
      prg_ram: vec![0u8; rom_header.prg_ram_len],
      chr_rom: vec![0u8; rom_header.chr_rom_len],
      chr_ram: vec![0u8; rom_header.chr_ram_len],
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mirroring {
  Vertical,
  Horizontal,
}

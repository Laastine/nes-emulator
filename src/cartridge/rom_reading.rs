use std::convert::TryFrom;
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
  pub tv_system: TVSystem,
  pub flag_bus_conflicts: bool,
}

#[derive(Clone, Debug)]
pub struct Rom {
  pub rom_header: RomHeader,
  pub prg_rom: Vec<u8>,
  pub chr_rom: Vec<u8>,
  pub title: String,
}

impl Rom {
  pub fn read_from_file(mut rom_bytes: impl Iterator<Item=u8>) -> Rom {
    let mut bytes = &mut rom_bytes;

    let first_4_bytes = (&mut bytes).take(4);
    if b"NES\x1A".iter().cloned().ne(first_4_bytes) {
      panic!("Invalid ROM header");
    }

    let prg_rom_size = bytes.next().unwrap_or_else(|| panic!("prg_rom read error"));
    let chr_rom_size = bytes.next().unwrap_or_else(|| panic!("chr_rom read error"));
    let flags_6 = bytes.next().unwrap_or_else(|| panic!("flags_6 read error"));
    let flags_7 = bytes.next().unwrap_or_else(|| panic!("flags_7 read error"));
    let prg_ram_size = bytes.next().unwrap_or_else(|| panic!("flags_8 read error"));
    let _flags_9 = bytes.next().unwrap_or_else(|| panic!("flags_9 read error"));
    let flags_10 = bytes
      .next()
      .unwrap_or_else(|| panic!("flags_10 read error"));

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
    let mapper_hi = (flags_7 & 0xF0) >> 4;

    let flag_tv_system = flags_10 & 0x03;
    let flag_prg_ram = (flags_10 & 0x10) > 0x00;
    let flag_bus_conflicts = (flags_10 & 0x20) > 0x00;

    if flag_rom_format == 2 {
      unimplemented!("NES 2.0 ROM format not implemented");
    }

    let prg_rom_len = prg_rom_size as usize * 0x4000;
    let chr_rom_len = chr_rom_size as usize * 0x2000;

    let prg_ram_len = usize::try_from(match (prg_ram_size, flag_prg_ram) {
      (_, false) => 0,
      (0, true) => 0x2000,
      (_, true) => prg_ram_size as usize * 0x2000,
    })
      .unwrap();

    let chr_ram_len = if chr_rom_size == 0 { 0x2000 } else { 0 };

    let mirroring = match (flag_mirror, flag_four_screen_vram) {
      (true, false) => Mirroring::Vertical,
      (false, false) => Mirroring::Horizontal,
      _ => panic!("Mirroring mode {}, {} not supported", flag_mirror, flag_four_screen_vram)
    };

    let mapper = mapper_lo | (mapper_hi << 4);

    let tv_system = TVSystem::get_tv_system(flag_tv_system);

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
      tv_system,
      flag_bus_conflicts,
    };

    let prg_rom = bytes.take(rom_header.prg_rom_len).collect::<Vec<u8>>();
    if prg_rom.len() != rom_header.prg_rom_len {
      panic!("Couldn't read PRG rom");
    }

    let chr_rom = bytes.take(rom_header.chr_rom_len).collect::<Vec<u8>>();
    if chr_rom.len() != rom_header.chr_rom_len {
      panic!("Couldn't read CHR rom");
    }

    let title_bytes = bytes.take(0x80).collect::<Vec<u8>>();
    let title = String::from_utf8_lossy(&title_bytes).to_string();

    if bytes.next().is_some() {
      panic!("Unexpected ROM size");
    }

    println!("Loading {}", title);

    Rom {
      rom_header,
      title,
      prg_rom,
      chr_rom,
    }
  }

  #[allow(dead_code)]
  pub fn mock_rom() -> Rom {
    let rom_header = RomHeader {
      prg_rom_len: 0x4000,
      chr_rom_len: 0x2000,
      prg_ram_len: 0x2000,
      chr_ram_len: 0x2000,
      mirroring: Mirroring::Horizontal,
      mapper: 0,
      flag_persistent: false,
      flag_trainer: false,
      flag_vs_unisystem: false,
      flag_playchoice_10: false,
      tv_system: TVSystem::NTSC,
      flag_bus_conflicts: false,
    };

    Rom {
      rom_header,
      prg_rom: vec![0u8; rom_header.prg_rom_len],
      chr_rom: vec![0u8; rom_header.chr_rom_len],
      title: "test".to_string(),
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub enum TVSystem {
  NTSC,
  PAL,
  DualCompatible,
}

impl TVSystem {
  pub fn get_tv_system(val: u8) -> TVSystem {
    match val {
      0 => TVSystem::NTSC,
      1 => TVSystem::PAL,
      3 => TVSystem::DualCompatible,
      _ => panic!("Unrecognized TV system value: {}", val),
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mirroring {
  Vertical,
  Horizontal,
}

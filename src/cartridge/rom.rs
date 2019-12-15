use std::convert::TryFrom;
use std::iter::Iterator;

#[derive(Clone)]
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

#[derive(Clone)]
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
    let flags_9 = bytes.next().unwrap_or_else(|| panic!("flags_9 read error"));
    let flags_10 = bytes.next().unwrap_or_else(|| panic!("flags_10 read error"));

    let zeros = (&mut bytes).take(5);
    if [0, 0, 0, 0, 0].iter().cloned().ne(zeros) {
      panic!("Non-zero bits found on unused block")
    }

    let flag_mirror = (flags_6 & 0b_0000_0001) != 0x00;
    let flag_persistent = (flags_6 & 0b_0000_0010) != 0x00;
    let flag_trainer = (flags_6 & 0b_0000_0100) != 0x00;
    let flag_four_screen_vram = (flags_6 & 0b_0000_1000) != 0x00;
    let mapper_lo = u8::try_from((flags_6 & 0b_1111_0000).wrapping_shr(4)).unwrap();

    let flag_vs_unisystem = (flags_7 & 0b_0000_0001) != 0x00;
    let flag_playchoice_10 = (flags_7 & 0b_0000_0010) != 0x00;
    let flag_rom_format = u8::try_from((flags_7 & 0b_0000_1100).wrapping_shr(2)).unwrap();
    let mapper_hi = u8::try_from((flags_7 & 0b_1111_0000).wrapping_shr(4)).unwrap();

    let flag_tv_system = flags_10 & 0b_0000_0011;
    let flag_prg_ram = (flags_10 & 0b_0001_0000) != 0x00;
    let flag_bus_conflicts = (flags_10 & 0b_0010_0000) != 0x00;

    if flag_rom_format == 2 {
      unimplemented!("NES 2.0 ROM format not implemented");
    }

    let prg_rom_len = prg_rom_size as usize * 0x4000;
    let chr_rom_len = chr_rom_size as usize * 0x2000;

    let prg_ram_len = usize::try_from(match (prg_ram_size, flag_prg_ram) {
      (_, false) => 0,
      (0, true) => 0x2000,
      (_, true) => prg_ram_size as usize * 0x2000,
    }).unwrap();

    let chr_ram_len = if chr_rom_size == 0 { 0x2000 } else { 0 };

    let mirroring = match (flag_mirror, flag_four_screen_vram) {
      (true, false) => Mirroring::Vertical,
      (false, false) => Mirroring::Horizontal,
      (_, true) => Mirroring::FourSreenVram,
    };

    let mapper = mapper_lo | mapper_hi.wrapping_shl(4);

    let tv_system = TVSystem::to_enum(flag_tv_system);

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
}

#[derive(Debug, Copy, Clone)]
pub enum TVSystem {
  NTSC,
  PAL,
  DualCompatible,
}

impl TVSystem {
  pub fn to_enum(value: u8) -> TVSystem {
    match value {
      0 => TVSystem::NTSC,
      1 => TVSystem::PAL,
      1 | 3 => TVSystem::DualCompatible,
      _ => panic!("Unrecognized TV system value: {}", value),
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub enum Mirroring {
  Vertical,
  Horizontal,
  FourSreenVram,
}

impl Mirroring {
  pub fn to_enum(value: u8) -> Mirroring {
    match value {
      0 => Mirroring::Vertical,
      1 => Mirroring::Horizontal,
      2 => Mirroring::FourSreenVram,
      _ => panic!("Unrecognized Mirroring value: {}", value),
    }
  }
}

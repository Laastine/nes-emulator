use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::cartridge::rom_reading::Mirroring;
use crate::cartridge::rom_with_pager::RomData;
use crate::mapper::Mapper;
use crate::mapper::pager::Page;
use crate::mapper::pager::PageSize::{Eight, One};

#[derive(Clone)]
pub(crate) struct Mapper4 {
  prg_select: bool,
  chr_select: bool,
  registers: [usize; 8],
  index: usize,
  mirroring: Mirroring,
  irq_counter: u8,
  irq_period: u8,
  irq_enabled: bool,
  flag_irq: bool,
  rom: Rc<RefCell<RomData>>,
}

impl Mapper4 {
  pub fn new(rom: Rc<RefCell<RomData>>) -> Mapper4 {
    Mapper4 {
      prg_select: false,
      chr_select: false,
      registers: [0; 8],
      index: 0,
      mirroring: Mirroring::Horizontal,
      irq_counter: 0,
      irq_period: 0,
      irq_enabled: false,
      flag_irq: false,
      rom,
    }
  }

  fn get_rom(&self) -> Ref<RomData> {
    self.rom.borrow()
  }

  fn get_mut_rom(&self) -> RefMut<RomData> {
    self.rom.borrow_mut()
  }
}

impl Mapper for Mapper4 {
  fn mapped_read_cpu_u8(&self, address: u16) -> u8 {
    match (address, self.prg_select) {
      (0x6000..=0x7FFF, _) => self.get_rom().prg_ram.read(Page::First(Eight), address - 0x6000),
      (0x8000..=0x9FFF, false) => self.get_rom().prg_rom.read(Page::FromNth(self.registers[6], Eight), address - 0x8000),
      (0x8000..=0x9FFF, true) => self.get_rom().prg_rom.read(Page::FromEnd(1, Eight), address - 0x8000),
      (0xA000..=0xBFFF, _) => self.get_rom().prg_rom.read(Page::FromNth(self.registers[7], Eight), address - 0xA000),
      (0xC000..=0xDFFF, false) => self.get_rom().prg_rom.read(Page::FromEnd(1, Eight), address - 0xC000),
      (0xC000..=0xDFFF, true) => self.get_rom().prg_rom.read(Page::FromNth(self.registers[6], Eight), address - 0xC000),
      (0xE000..=0xFFFF, _) => self.get_rom().prg_rom.read(Page::FromEnd(0, Eight), address - 0xE000),
      _ => panic!("Invalid mapped_read_cpu_u8 address 0x{:04X}", address),
    }
  }

  fn mapped_write_cpu_u8(&mut self, address: u16, data: u8) {
    match (address, address % 2) {
      (0x6000..=0x7FFF, _) => self.get_mut_rom().prg_ram.write(Page::First(Eight), address - 0x6000, data),
      (0x8000..=0x9FFF, 0) => {
        self.index = data as usize & 0x07;
        self.prg_select = data & 0x40 > 0;
        self.chr_select = data & 0x80 > 0;
      }
      (0x8000..=0x9FFF, 1) => {
        self.registers[self.index] = data as usize;
      }
      (0xA000..=0xBFFF, 0) => {
        self.mirroring = if data % 2 == 0 { Mirroring::Vertical } else { Mirroring::Horizontal };
      }
      (0xC000..=0xDFFF, 0) => self.irq_period = data,
      (0xC000..=0xDFFF, 1) => self.irq_counter = 0,
      (0xE000..=0xFFFF, 0) => {
        self.irq_enabled = false;
        self.flag_irq = false;
      }
      (0xE000..=0xFFFF, 1) => self.irq_enabled = true,
      _ => (),
    }
  }

  fn mapped_read_ppu_u8(&self, address: u16) -> u8 {
    let bank = match (address, self.chr_select) {
      (0x0000..=0x03FF, false) => self.registers[0] & !1,
      (0x0000..=0x03FF, true) => self.registers[2],
      (0x0400..=0x07FF, false) => self.registers[0] | 1,
      (0x0400..=0x07FF, true) => self.registers[3],
      (0x0800..=0x0BFF, false) => self.registers[1] & !1,
      (0x0800..=0x0BFF, true) => self.registers[4],
      (0x0C00..=0x0FFF, false) => self.registers[1] | 1,
      (0x0C00..=0x0FFF, true) => self.registers[5],

      (0x1000..=0x13FF, false) => self.registers[2],
      (0x1000..=0x13FF, true) => self.registers[0] & !1,
      (0x1400..=0x17FF, false) => self.registers[3],
      (0x1400..=0x17FF, true) => self.registers[0] | 1,
      (0x1800..=0x1BFF, false) => self.registers[4],
      (0x1800..=0x1BFF, true) => self.registers[1] & !1,
      (0x1C00..=0x1FFF, false) => self.registers[5],
      (0x1C00..=0x1FFF, true) => self.registers[1] | 1,
      _ => panic!("Invalid mapped_read_ppu_u8 address 0x{:04X}", address),
    };
    self.get_rom().chr_rom.read(Page::FromNth(bank, One), address & 0x03FF)
  }

  fn mapped_write_ppu_u8(&mut self, _address: u16, _data: u8) {}

  fn mirroring(&self) -> Mirroring {
    self.mirroring
  }

  fn irq_flag(&self) -> bool {
    self.flag_irq
  }

  fn signal_scanline(&mut self) {
    if self.irq_counter == 0 {
      self.irq_counter = self.irq_period;
    } else {
      self.irq_counter -= 1;
    }
    if self.irq_counter == 0 && self.irq_enabled {
      self.flag_irq = true;
    }
  }

  fn clear_irq_flag(&mut self) {
    self.flag_irq = false;
  }
}

use crate::mapper::Mapper;
use crate::cartridge::rom_reading::Mirroring;
use crate::mapper::pager::{PageSize, Page};
use crate::cartridge::rom_with_pager::RomData;
use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum AddressRange {
  Lo,
  Hi,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum PrgMode {
  Consecutive,
  FixFirst,
  FixLast,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ChrMode {
  Consecutive,
  NonConsecutive,
}

bitfield!{
    #[derive(Copy, Clone)]
    struct CtrlReg(u8);
    impl Debug;
    nt_mode_id,   _: 1,  0;
    prg_mode_id,  _: 3,  2;
    chr_mode_id,  _: 4,  4;
}

impl CtrlReg {
  fn mirroring(&self) -> Mirroring {
    match self.nt_mode_id() {
      2 => Mirroring::Vertical,
      3 => Mirroring::Horizontal,
      _ => panic!("Invalid mirroring mode"),
    }
  }

  fn prg_mode(&self) -> PrgMode {
    match self.prg_mode_id() {
      0 | 1 => PrgMode::Consecutive,
      2 => PrgMode::FixFirst,
      3 => PrgMode::FixLast,
      _ => panic!("prg mode error"),
    }
  }

  fn chr_mode(&self) -> ChrMode {
    match self.chr_mode_id() {
      0 => ChrMode::Consecutive,
      1 => ChrMode::NonConsecutive,
      _ => panic!("chr mode error"),
    }
  }
}

#[derive(Clone)]
struct ShiftReg {
  val: u8,
  idx: u8,
}

impl ShiftReg {
  fn new() -> Self {
    ShiftReg {
      val: 0,
      idx: 0,
    }
  }
  fn reset(&mut self) {
    self.val = 0;
    self.idx = 0;
  }

  fn push(&mut self, n: u8) -> Option<u8> {
    if n >> 7 == 1 {
      self.reset();
    } else {
      self.val |= (n & 1) << self.idx;
      if self.idx == 4 {
        let result = self.val;
        self.reset();
        return Some(result);
      }
      self.idx += 1;
    }
    None
  }
}

#[derive(Clone)]
pub(crate) struct Mapper1 {
  rom: Rc<RefCell<RomData>>,
  shift_reg: ShiftReg,
  control_reg: CtrlReg,
  prg_0: usize,
  chr_0: usize,
  chr_1: usize,
}

impl Mapper1 {
  pub fn new(rom: Rc<RefCell<RomData>>) -> Self {
    Mapper1 {
      rom,
      shift_reg: ShiftReg::new(),
      control_reg: CtrlReg(0x0E),
      chr_0: 0,
      chr_1: 0,
      prg_0: 0,
    }
  }

  fn get_rom(&self) -> Ref<RomData> {
    self.rom.borrow()
  }

  fn get_mut_rom(&self) -> RefMut<RomData> {
    self.rom.borrow_mut()
  }

  fn write_shift(&mut self, address: u16, value: u8) {
    if let Some(shift_value) = self.shift_reg.push(value) {
      match address {
        0x8000..=0x9FFF => self.control_reg = CtrlReg(shift_value),
        0xA000..=0xBFFF => self.chr_0 = shift_value as usize & 0x1F,
        0xC000..=0xDFFF => self.chr_1 = shift_value as usize & 0x1F,
        0xE000..=0xFFFF => self.prg_0 = shift_value as usize & 0x0F,
        _ => panic!("Invalid write_shift address 0x{:04X}", address),
      }
    }
  }

  fn read_paged_prg_ram(&self, offset: u16) -> u8 {
    self.get_rom()
      .prg_ram
      .read(Page::First(PageSize::Eight), offset)
  }

  fn write_paged_prg_ram(&mut self, offset: u16, value: u8) {
    self.get_mut_rom()
      .prg_ram
      .write(Page::First(PageSize::Eight), offset, value);
  }

  fn get_page(&self, address_range: AddressRange) -> Page {
    match self.control_reg.chr_mode() {
      ChrMode::Consecutive => match address_range {
        AddressRange::Lo => Page::FromNth(self.chr_0, PageSize::Four),
        AddressRange::Hi => Page::FromNth(self.chr_0 + 1, PageSize::Four),
      },
      ChrMode::NonConsecutive => match address_range {
        AddressRange::Lo => Page::FromNth(self.chr_0, PageSize::Four),
        AddressRange::Hi => Page::FromNth(self.chr_1, PageSize::Four),
      },
    }
  }

  fn write_paged_chr_ram(&mut self, address_range: AddressRange, offset: u16, value: u8) {
    self.get_mut_rom().chr_ram.write(self.get_page(address_range), offset, value)
  }

  fn read_paged_prg_rom(&self, address_range: AddressRange, offset: u16) -> u8 {
    let page = match self.control_reg.prg_mode() {
      PrgMode::FixFirst => match address_range {
        AddressRange::Lo => Page::First(PageSize::Sixteen),
        AddressRange::Hi => Page::FromNth(self.prg_0, PageSize::Sixteen),
      },
      PrgMode::FixLast => match address_range {
        AddressRange::Lo => Page::FromNth(self.prg_0, PageSize::Sixteen),
        AddressRange::Hi => Page::Last(PageSize::Sixteen),
      },
      PrgMode::Consecutive => match address_range {
        AddressRange::Lo => Page::FromNth(self.prg_0 & !1, PageSize::Sixteen),
        AddressRange::Hi => Page::FromNth(self.prg_0 | 1, PageSize::Sixteen),
      },
    };
    self.get_rom().prg_rom.read(page, offset)
  }

  fn read_paged_chr_rom(&self, address_range: AddressRange, offset: u16) -> u8 {
    let page = self.get_page(address_range);

    if self.get_mut_rom().rom_header.chr_rom_len == 0 {
      self.get_rom().chr_ram.read(page, offset)
    } else {
      self.get_rom().chr_rom.read(page, offset)
    }
  }
}

impl Mapper for Mapper1 {
  fn mapped_read_cpu_u8(&self, address: u16) -> u8 {
    match address {
      0x6000..=0x7FFF => self.read_paged_prg_ram(address - 0x6000),
      0x8000..=0xBFFF => self.read_paged_prg_rom(AddressRange::Lo, address - 0x8000),
      0xC000..=0xFFFF => self.read_paged_prg_rom(AddressRange::Hi, address - 0xC000),
      _ => panic!("Invalid mapped_read_cpu_u8 address 0x{:04X}", address),
    }
  }

  fn mapped_write_cpu_u8(&mut self, address: u16, value: u8) {
    match address {
      0x6000..=0x7FFF => self.write_paged_prg_ram(address - 0x6000, value),
      0x8000..=0xFFFF => self.write_shift(address, value),
      _ => panic!("Invalid mapped_write_cpu_u8 address 0x{:04X}", address),
    }
  }

  fn mapped_read_ppu_u8(&self, address: u16) -> u8 {
    match address {
      0x0000..=0x0FFF => self.read_paged_chr_rom(AddressRange::Lo, address),
      0x1000..=0x1FFF => self.read_paged_chr_rom(AddressRange::Hi, address - 0x1000),
      _ => panic!("Invalid mapped_read_ppu_u8 address 0x{:04X}", address),
    }
  }

  fn mapped_write_ppu_u8(&mut self, address: u16, value: u8) {
    match address {
      0x0000..=0x0FFF => self.write_paged_chr_ram(AddressRange::Lo, address, value),
      0x1000..=0x1FFF => {
        self.write_paged_chr_ram(AddressRange::Hi, address - 0x1000, value)
      }
      _ => panic!("Invalid mapped_write_ppu_u8 address 0x{:04X}", address),
    }
  }

  fn mirroring(&self) -> Mirroring {
    self.control_reg.mirroring()
  }
}

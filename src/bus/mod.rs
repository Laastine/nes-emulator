use std::convert::TryFrom;

use crate::cartridge::Cartridge;
use crate::mapper::Mapper;
use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use crate::ppu::registers::{Registers, PpuCtrlFlags, PpuMaskFlags, PpuStatusFlags, ScrollRegister};

pub const MEM_SIZE: usize = 0x0800;

#[derive(Clone)]
pub struct Bus {
  pub cartridge: Cartridge,
  pub ram: [u8; MEM_SIZE],
  mapper: Mapper,
  controller: [u8; 2],
  registers: Rc<RefCell<Registers>>
}

impl Bus {
  pub fn new(cartridge: Cartridge, mapper: Mapper, registers: Rc<RefCell<Registers>>) -> Bus {
    let ram = [0u8; MEM_SIZE];
    let controller = [0u8; 2];

    Bus {
      cartridge,
      mapper,
      ram,
      controller,
      registers
    }
  }

  pub fn get_mut_registers(&mut self) -> RefMut<Registers> {
    self.registers.borrow_mut()
  }

  pub fn write_u8(&mut self, address: u16, data: u8) {
    match address {
      0x0000..=0x1FFF => {
        self.ram[usize::try_from(address & 0x07FF).unwrap()] = data;
      }
      0x2000..=0x3FFF => {
        self.write_ppu_registers(address, data)
      }
      0x8000..=0xFFFF => {
        let mapped_addr = usize::try_from(self.mapper.write_cpu_u8(address)).unwrap();
        {
          let prg_rom = self.cartridge.get_prg_rom();
          prg_rom[mapped_addr] = data
        };
      }
      _ => (),
    }
  }

  fn write_ppu_registers(&mut self, address: u16, data: u8) {
    match address {
      0x00 => {
        self.get_mut_registers().ctrl_flags = PpuCtrlFlags::from_bits_truncate(data);
        let tram_addr = self.get_mut_registers().tram_addr.bits();
        let ctrl_flags = self.get_mut_registers().ctrl_flags.bits();
        self.get_mut_registers().tram_addr = ScrollRegister::from_bits_truncate((tram_addr + u16::try_from(ctrl_flags & 0x03).unwrap()).into());
      },
      0x01 => {
        self.get_mut_registers().mask_flags = PpuMaskFlags::from_bits_truncate(data)
      },
      0x02 => {},
      0x03 => {},
      0x04 => {},
      0x05 => { // Scroll
        if self.get_mut_registers().address_latch == 0 { // X
          self.get_mut_registers().fine_x = data & 0x07;
          self.get_mut_registers().tram_addr = ScrollRegister::from_bits_truncate(data.wrapping_shr(3).into());
          self.get_mut_registers().address_latch = 1;
        } else {                    // Y
          self.get_mut_registers().tram_addr = ScrollRegister::from_bits_truncate(((data & 0x07) + (data.wrapping_shr(3))).into());
          self.get_mut_registers().address_latch = 0;
        }
      },
      0x06 => { // PPU address
        let tram_addr = self.get_mut_registers().tram_addr.bits();
        if self.get_mut_registers().address_latch == 0 {
          self.get_mut_registers().tram_addr = ScrollRegister::from_bits_truncate(u16::try_from(data & 0x3F).unwrap() | tram_addr & 0x00FF);
          self.get_mut_registers().address_latch = 1;
        } else {
          self.get_mut_registers().tram_addr = ScrollRegister::from_bits_truncate(tram_addr & 0xFF00 | u16::try_from(data).unwrap());
          let new_tram_addr = self.get_mut_registers().tram_addr;
          self.get_mut_registers().vram_addr = new_tram_addr;
          self.get_mut_registers().address_latch = 0;
        }
      },
      0x07 => {
        let vram_addr = self.get_mut_registers().vram_addr.bits();
        self.write_u8(vram_addr, data);
        let vram_addr = self.get_mut_registers().vram_addr.bits();
        let val = if (self.get_mut_registers().ctrl_flags.bits() & 0x04) > 0x00 { 32 } else { 1 };
        self.get_mut_registers().vram_addr = ScrollRegister::from_bits_truncate(vram_addr + val);
      },
      _ => {},
    };
  }

  pub fn read_u8(&mut self, address: u16) -> u16 {
    match address {
      0x0000..=0x1FFF => {
        let idx = usize::try_from(address & 0x07FF).unwrap();
        u16::try_from(self.ram[idx]).unwrap()
      }
      0x2000..=0x3FFF => {
        self.read_ppu_u8(address, true).into()
      },
      0x4016..=0x4016 => {
        let res = if (self.controller[usize::try_from(address & 0x0001).unwrap()] & 0x80) > 0 { 1u8 } else { 0u8 };
        self.controller[usize::try_from(address & 0x0001).unwrap()] <<= 1;
        res.into()
      }
      0x8000..=0xFFFF => {
        let mapped_addr = usize::try_from(self.mapper.read_cpu_u8(address)).unwrap();
        u16::try_from({
          let prg_rom = self.cartridge.get_prg_rom();
          prg_rom[mapped_addr]
        })
        .unwrap()
      }
      _ => 0,
    }
  }

  fn read_ppu_u8(&mut self, address: u16, read_only: bool) -> u8 {
    let addr = if read_only {
      match address {
        0x00 => self.get_mut_registers().ctrl_flags.bits(),
        0x01 => self.get_mut_registers().mask_flags.bits(),
        0x02 => self.get_mut_registers().status_flags.bits(),
        0x03 => 0x00,
        0x04 => 0x00,
        0x05 => 0x00,
        0x06 => 0x00,
        0x07 => 0x00,
        0x08 => 0x00,
        _ => 0x00,
      }
    } else {
      match address {
        0x00 => 0x00,
        0x01 => 0x00,
        0x02 => {
          self.get_mut_registers().status_flags = PpuStatusFlags::from_bits_truncate(0);
          self.get_mut_registers().address_latch = 0x00;
          let status_flags = self.get_mut_registers().status_flags.bits();
          (status_flags & 0xE0 | self.get_mut_registers().ppu_data_buffer & 0x1F)
        }
        0x03 => 0x00,
        0x04 => 0x00,
        0x05 => 0x00,
        0x06 => 0x00,
        0x07 => {
          let mut data = self.get_mut_registers().ppu_data_buffer;
          let vram_addr = self.get_mut_registers().vram_addr.bits();
          self.get_mut_registers().ppu_data_buffer = u8::try_from(self.read_u8(vram_addr)).unwrap();
          if self.get_mut_registers().vram_addr.bits() >= 0x3F00 {
            data = self.get_mut_registers().ppu_data_buffer;
          }
          let vram_addr = self.get_mut_registers().vram_addr.bits();
          let val = if (self.get_mut_registers().ctrl_flags.bits() & 0x04) > 0x00 { 32 } else { 1 };
          self.get_mut_registers().vram_addr = ScrollRegister::from_bits_truncate(vram_addr + val);
          data
        }
        _ => 0x00,
      }
    };
    u8::try_from(self.read_u8(u16::try_from(addr).unwrap())).unwrap()
  }
}

use std::cell::{RefCell, RefMut};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::cartridge::Cartridge;
use crate::ppu::registers::{PpuCtrlFlags, PpuMaskFlags, Registers, ScrollRegister};

pub const MEM_SIZE: usize = 0x0800;

#[derive(Clone)]
pub struct Bus {
  pub cartridge: Rc<RefCell<Cartridge>>,
  pub ram: [u8; MEM_SIZE],
  controller: [u8; 2],
  registers: Rc<RefCell<Registers>>
}

impl Bus {
  pub fn new(cartridge: Rc<RefCell<Cartridge>>, registers: Rc<RefCell<Registers>>, ) -> Bus {
    let ram = [0u8; MEM_SIZE];
    let controller = [0u8; 2];

    Bus {
      cartridge,
      ram,
      controller,
      registers
    }
  }

  pub fn get_mut_cartridge(&mut self) -> RefMut<Cartridge> {
    self.cartridge.borrow_mut()
  }

  pub fn get_mut_registers(&mut self) -> RefMut<Registers> {
    self.registers.borrow_mut()
  }

  pub fn write_u8(&mut self, address: u16, data: u8) {
    let (is_address_in_range, mapped_addr) = self.get_mut_cartridge().mapper.mapped_read_cpu_u8(address);
    if is_address_in_range {
      self.get_mut_cartridge().rom.prg_rom[mapped_addr] = data;
    } else {
      match address {
        0x0000..=0x1FFF => {
          self.ram[usize::try_from(address & 0x07FF).unwrap()] = data;
        }
        0x2000..=0x3FFF => {
          self.get_mut_registers().write_ppu_registers(address & 0x0007, data)
        }
        0x4016..=0x4017 => {
          let idx = usize::try_from(address & 0x0001).unwrap();
          self.controller[idx] = self.controller[idx];
        }
        _ => (),
      }
    }
  }

  pub fn read_u8(&mut self, address: u16, read_only: bool) -> u16 {
    let (is_address_in_range, mapped_addr) = self.get_mut_cartridge().mapper.mapped_read_cpu_u8(address);
    if is_address_in_range {
      let res = u16::try_from(self.get_mut_cartridge().rom.prg_rom[mapped_addr]).unwrap();
//      println!("A PPU data: {} -> {}", address, res);
      res
    } else {
      match address {
        0x0000..=0x1FFF => {
          let res= self.ram[usize::try_from(address & 0x07FF).unwrap()].into();
//          println!("B PPU data: {} -> {}", address, res);
          res
        }
        0x2000..=0x3FFF => {
          let res = self.get_mut_registers().read_ppu_registers(address & 0x0007, read_only).into();
//          println!("C PPU data: {} -> {}", address, res);
          res
        },
        0x4016..=0x4017 => {
          let res: u16 = if (self.controller[usize::try_from(address & 0x0001).unwrap()] & 0x80) > 0x00 { 1 } else { 0 };
          self.controller[usize::try_from(address & 0x0001).unwrap()] <<= 1;
//          println!("D PPU data: {} -> {}", address, res);
          res
        }
        _ => 0,
      }
    }
  }
}

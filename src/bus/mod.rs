use std::cell::{RefCell, RefMut, Ref};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::cartridge::Cartridge;
use crate::ppu::registers::Registers;

pub const MEM_SIZE: usize = 0x0800;

#[derive(Clone)]
pub struct Bus {
  pub cartridge: Rc<RefCell<Cartridge>>,
  pub ram: [u8; MEM_SIZE],
  controller: Rc<RefCell<[u8; 2]>>,
  controller_state: [u8; 2],
  registers: Rc<RefCell<Registers>>,
}

impl Bus {
  pub fn new(cartridge: Rc<RefCell<Cartridge>>, registers: Rc<RefCell<Registers>>, controller: Rc<RefCell<[u8; 2]>>) -> Bus {
    let ram = [0u8; MEM_SIZE];
    let controller_state = [0u8; 2];

    Bus {
      cartridge,
      ram,
      controller,
      controller_state,
      registers,
    }
  }

  fn get_controller(&mut self) -> Ref<[u8; 2]> {
    self.controller.borrow()
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
    } else if (0x0000..=0x1FFF).contains(&address) {
      self.ram[usize::try_from(address & 0x07FF).unwrap()] = data;
    } else if (0x2000..=0x3FFF).contains(&address) {
      self.get_mut_registers().cpu_write(address & 0x0007, data)
    } else if (0x4016..=0x4017).contains(&address) {
      let idx = usize::try_from(address & 1).unwrap();
      let new_controller_state = self.get_controller()[idx];
      self.controller_state[idx] = new_controller_state;
    }
  }

  pub fn read_u8(&mut self, address: u16, read_only: bool) -> u16 {
    let (is_address_in_range, mapped_addr) = self.get_mut_cartridge().mapper.mapped_read_cpu_u8(address);
    if is_address_in_range {
      u16::try_from(self.get_mut_cartridge().rom.prg_rom[mapped_addr]).unwrap()
    } else if (0x0000..=0x1FFF).contains(&address) {
      u16::try_from(self.ram[usize::try_from(address).unwrap() & 0x07FF]).unwrap()
    } else if (0x2000..=0x3FFF).contains(&address) {
      self.get_mut_registers().cpu_read(address & 0x0007, read_only).into()
    } else if (0x4016..=0x4017).contains(&address) {
      let idx = usize::try_from(address & 0x0001).unwrap();
      let state = self.controller_state[idx] & 0x80;
      self.controller_state[idx] <<= 1;
      if state > 0x00 { 1 } else { 0 }
    } else {
      0
    }
  }
}

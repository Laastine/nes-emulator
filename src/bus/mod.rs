use std::cell::{Ref, RefCell, RefMut};
use std::convert::{TryFrom, TryInto};
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
  pub dma_transfer: bool,
  dma_page: u8,
  dma_data: u8,
  is_odd_cycle: bool,
}

impl Bus {
  pub fn new(cartridge: Rc<RefCell<Cartridge>>, registers: Rc<RefCell<Registers>>, controller: Rc<RefCell<[u8; 2]>>) -> Bus {
    let ram = [0u8; MEM_SIZE];
    let controller_state = [0u8; 2];
    let dma_transfer = false;
    let dma_page = 0x00;
    let dma_data = 0x00;

    Bus {
      cartridge,
      ram,
      controller,
      controller_state,
      registers,
      dma_transfer,
      dma_page,
      dma_data,
      is_odd_cycle: true,
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
    } else if address == 0x4014 {
      self.dma_page = data;
      self.get_mut_registers().oam_address = 0x00;
      self.dma_transfer = true;
    } else if (0x4016..=0x4017).contains(&address) {
      let idx = usize::try_from(address & 1).unwrap();
      let new_controller_state = self.get_controller()[idx];
      self.controller_state[idx] = new_controller_state;
    }
  }

  pub fn read_u8(&mut self, address: u16) -> u16 {
    let (is_address_in_range, mapped_addr) = self.get_mut_cartridge().mapper.mapped_read_cpu_u8(address);
    if is_address_in_range {
      u16::try_from(self.get_mut_cartridge().rom.prg_rom[mapped_addr]).unwrap()
    } else if (0x0000..=0x1FFF).contains(&address) {
      u16::try_from(self.ram[usize::try_from(address).unwrap() & 0x07FF]).unwrap()
    } else if (0x2000..=0x3FFF).contains(&address) {
      self.get_mut_registers().cpu_read(address & 0x0007).into()
    } else if (0x4016..=0x4017).contains(&address) {
      let idx = usize::try_from(address & 0x0001).unwrap();
      let state = self.controller_state[idx] & 0x80;
      self.controller_state[idx] <<= 1;
      if state > 0x00 { 1 } else { 0 }
    } else {
      0
    }
  }

  pub fn oam_dma_access(&mut self, system_cycles: u64) {
    if self.is_odd_cycle {
      if system_cycles % 2 == 1 {
        self.is_odd_cycle = false;
      }
    } else if system_cycles % 2 == 0 {
      let oam_address = u16::try_from(self.get_mut_registers().oam_address).unwrap();
      self.dma_data = self.read_u8(u16::try_from(u16::try_from(self.dma_page).unwrap().wrapping_shl(8) | oam_address).unwrap()).try_into().unwrap();
    } else {
      let dma_data = self.dma_data;
      self.get_mut_registers().write_oam_data(dma_data);

      if self.get_mut_registers().oam_address == 0 {
        self.dma_transfer = false;
        self.is_odd_cycle = true;
      }
    }
  }
}

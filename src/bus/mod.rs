use std::cell::{Ref, RefCell, RefMut};
use std::convert::TryInto;
use std::rc::Rc;

use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::nes::controller::Controller;
use crate::ppu::registers::Registers;

pub const MEM_SIZE: usize = 0x0800;

#[derive(Clone)]
pub struct Bus {
  pub cartridge: Rc<RefCell<Box<Cartridge>>>,
  pub ram: [u8; MEM_SIZE],
  apu: Rc<RefCell<Apu>>,
  controller: Rc<RefCell<Controller>>,
  registers: Rc<RefCell<Registers>>,
  pub dma_transfer: bool,
  dma_page: u8,
}

impl Bus {
  pub fn new(cartridge: Rc<RefCell<Box<Cartridge>>>, registers: Rc<RefCell<Registers>>, controller: Rc<RefCell<Controller>>, apu: Rc<RefCell<Apu>>) -> Bus {
    let ram = [0u8; MEM_SIZE];
    let dma_transfer = false;
    let dma_page = 0x00;

    Bus {
      cartridge,
      ram,
      apu,
      controller,
      registers,
      dma_transfer,
      dma_page,
    }
  }

  fn get_controller(&mut self) -> RefMut<Controller> {
    self.controller.borrow_mut()
  }

  pub fn get_mut_apu(&mut self) -> RefMut<Apu> {
    self.apu.borrow_mut()
  }

  pub fn get_mut_cartridge(&mut self) -> RefMut<Box<Cartridge>> {
    self.cartridge.borrow_mut()
  }

  pub fn get_cartridge(&self) -> Ref<Box<Cartridge>> {
    self.cartridge.borrow()
  }

  pub fn get_mut_registers(&mut self) -> RefMut<Registers> {
    self.registers.borrow_mut()
  }

  pub fn write_u8(&mut self, address: u16, data: u8, cycles: u32) {
    if (0x0000..=0x1FFF).contains(&address) {
      self.ram[usize::from(address & 0x07FF)] = data;
    } else if (0x2000..=0x3FFF).contains(&address) {
      self.get_mut_registers().bus_write_ppu_reg(address, data)
    } else if address == 0x4014 {
      self.dma_page = data;
      self.get_mut_registers().oam_address = 0x00;
      self.dma_transfer = true;
    } else if (0x4000..=0x4013).contains(&address) || 0x4015 == address {
      self.get_mut_apu().apu_write_reg(address, data, cycles);
    } else if 0x4016 == address {
      self.get_controller().write(data);
    } else if 0x4017 == address {
      self.get_mut_apu().apu_write_reg(address, data, cycles);
    } else if (0x6000..=0xFFFF).contains(&address) {
      self.get_mut_cartridge().mapper.mapped_write_cpu_u8(address, data);
    }
  }

  pub fn read_u8(&mut self, address: u16) -> u8 {
    if (0x0000..=0x1FFF).contains(&address) {
      self.ram[usize::from(address & 0x07FF)]
    } else if (0x2000..=0x3FFF).contains(&address) {
      self.get_mut_registers().bus_read_ppu_reg(address)
    } else if address == 0x4015 {
      self.get_mut_apu().apu_read_reg()
    } else if 0x4016 == address {
      self.get_controller().read()
    } else if 0x4017 == address {
      0
    } else if (0x6000..=0xFFFF).contains(&address) {
      self.get_cartridge().mapper.mapped_read_cpu_u8(address)
    } else {
      address.try_into().unwrap()
    }
  }

  pub fn read_dbg_u8(&mut self, address_start: usize, address_end: usize) -> Vec<u8> {
    if (0x0000..=0x1FFF).contains(&address_start) && (0x0000..=0x1FFF).contains(&address_end) {
      return self.ram[address_start .. address_end].to_vec()
    }
    vec![]
  }

  pub fn oam_dma_access(&mut self, system_cycles: u32) -> u32 {
    let cpu_dma_cycles = 513 + (system_cycles % 2);
    for idx in 0..=255 {
      let addr = (u16::from(self.dma_page) << 8) + idx;
      let dma_data = self.read_u8(addr);
      self.get_mut_registers().write_oam_data(dma_data);
    }
    self.dma_transfer = false;
    cpu_dma_cycles
  }
}

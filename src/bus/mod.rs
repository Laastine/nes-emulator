use std::cell::{Ref, RefCell, RefMut};
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::ppu::registers::Registers;

pub const MEM_SIZE: usize = 0x0800;

#[derive(Clone)]
pub struct Bus {
  pub cartridge: Rc<RefCell<Box<Cartridge>>>,
  pub ram: [u8; MEM_SIZE],
  apu: Rc<RefCell<Apu>>,
  controller: Rc<RefCell<[bool; 8]>>,
  registers: Rc<RefCell<Registers>>,
  pub dma_transfer: bool,
  dma_page: u8,
  pub stall_cycles: u32,
  strobe: u8,
  idx: usize
}

impl Bus {
  pub fn new(cartridge: Rc<RefCell<Box<Cartridge>>>, registers: Rc<RefCell<Registers>>, controller: Rc<RefCell<[bool; 8]>>, apu: Rc<RefCell<Apu>>) -> Bus {
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
      stall_cycles: 0,
      strobe: 0,
      idx: 0,
    }
  }

  fn get_controller(&mut self) -> Ref<[bool; 8]> {
    self.controller.borrow()
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
      self.ram[usize::try_from(address & 0x07FF).unwrap()] = data;
    } else if (0x2000..=0x3FFF).contains(&address) {
      self.get_mut_registers().bus_write_ppu_reg(address, data)
    } else if address == 0x4014 {
      self.dma_page = data;
      self.get_mut_registers().oam_address = 0x00;
      self.dma_transfer = true;
    } else if (0x4000..=0x4013).contains(&address) || 0x4015 == address {
      self.get_mut_apu().apu_write_reg(address, data, cycles);
    } else if 0x4016 == address {
      self.strobe = data;

      if self.strobe & 1 == 1 {
        self.idx = 0;
      }
    } else if 0x4017 == address {
      self.get_mut_apu().apu_write_reg(address, data, cycles);
    } else if (0x4018..=0xFFFF).contains(&address) {
      self.get_mut_cartridge().mapper.mapped_write_cpu_u8(address, data);
    }
  }

  pub fn read_u8(&mut self, address: u16) -> u16 {
    if (0x0000..=0x1FFF).contains(&address) {
      u16::try_from(self.ram[usize::try_from(address & 0x07FF).unwrap()]).unwrap()
    } else if (0x2000..=0x3FFF).contains(&address) {
      self.get_mut_registers().bus_read_ppu_reg(address).into()
    } else if address == 0x4015 {
      u16::try_from(self.get_mut_apu().apu_read_reg()).unwrap()
    } else if 0x4016 == address {
      let idx = self.idx;
      let state = if self.get_controller()[idx] { 1 } else { 0 };

      self.idx += 1;
      if self.strobe & 1 == 1 {
        self.idx = 0;
      }
      state
    } else if 0x4017 == address {
      0
    } else if (0x4018..=0xFFFF).contains(&address) {
      u16::try_from(self.get_cartridge().mapper.mapped_read_cpu_u8(address)).unwrap()
    } else {
      address >> 8
    }
  }

  pub fn oam_dma_access(&mut self, system_cycles: u32) -> u32 {
    let cpu_dma_cycles = 513 + (system_cycles % 2);
    for idx in 0..=255 {
      let addr = (u16::try_from(self.dma_page).unwrap() << 8) + idx;
      let dma_data = self.read_u8(addr).try_into().unwrap();
      self.get_mut_registers().write_oam_data(dma_data);
    }
    self.dma_transfer = false;
    cpu_dma_cycles
  }
}

use std::cell::{Ref, RefCell, RefMut};
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

use crate::apu::Apu;
use crate::bus::interrupt::Interrupt;
use crate::cartridge::Cartridge;
use crate::nes::controller::Controller;
use crate::ppu::{Ppu, PpuState};

pub const MEM_SIZE: usize = 0x0800;

mod interrupt;

#[derive(Clone)]
pub struct Bus {
  pub cartridge: Rc<RefCell<Box<Cartridge>>>,
  pub cycles: u32,
  pub ram: [u8; MEM_SIZE],
  apu: Rc<RefCell<Apu>>,
  ppu: Rc<RefCell<Ppu>>,
  controller: Rc<RefCell<Controller>>,
  nmi: Interrupt,
  pub dma_transfer: bool,
  dma_page: u8,
  render: bool,
}

impl Bus {
  pub fn new(cartridge: Rc<RefCell<Box<Cartridge>>>, controller: Rc<RefCell<Controller>>, ppu: Rc<RefCell<Ppu>>, apu: Rc<RefCell<Apu>>) -> Bus {
    let ram = [0u8; MEM_SIZE];
    let dma_transfer = false;
    let render = false;
    let dma_page = 0x00;
    let nmi = Interrupt::new();
    let cycles = 0;

    Bus {
      cartridge,
      cycles,
      ram,
      ppu,
      apu,
      controller,
      nmi,
      dma_transfer,
      dma_page,
      render,
    }
  }

  pub fn tick(&mut self) {
    self.cycles += 1;
    let c = self.cycles;
    self.get_mut_apu().tick(c);
    // TODO: siirrä nes/mod.rs cycles logiikkaa bus tick funktioon.
    self.nmi.tick();
    let r = self.get_mut_ppu().tick();
    self.handle_ppu_result(r);
    let r =  self.get_mut_ppu().tick();
    self.handle_ppu_result(r);
    let r =  self.get_mut_ppu().tick();
    self.handle_ppu_result(r);
  }

  fn handle_ppu_result(&mut self, result: PpuState) {
    match result {
      PpuState::NonMaskableInterrupt => {
        self.nmi.schedule(1);
      }
      PpuState::Scanline =>  {
        self.get_mut_cartridge().signal_scanline();
      },
      PpuState::Render => {
        self.render = true;
      }
      PpuState::NoOp => {}
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

  pub fn get_mut_ppu(&mut self) -> RefMut<Ppu> {
    self.ppu.borrow_mut()
  }

  pub fn write_u8_with_tick(&mut self, address: u16, data: u8) {
    self.tick();
    self.write_u8(address, data);
  }

  pub fn write_u8(&mut self, address: u16, data: u8) {
    let cycles = self.cycles;
    if (0x0000..=0x1FFF).contains(&address) {
      self.ram[usize::try_from(address & 0x07FF).unwrap()] = data;
    } else if (0x2000..=0x3FFF).contains(&address) {
      self.get_mut_ppu().get_mut_registers().bus_write_ppu_reg(address, data)
    } else if address == 0x4014 {
      self.dma_page = data;
      self.get_mut_ppu().get_mut_registers().oam_address = 0x00;
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
      self.ram[usize::try_from(address & 0x07FF).unwrap()]
    } else if (0x2000..=0x3FFF).contains(&address) {
      self.get_mut_ppu().get_mut_registers().bus_read_ppu_reg(address)
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

  pub fn read_byte<T: Into<u16>>(&mut self, address: T) -> u8 {
    self.tick();
    self.read_u8(address.into())
  }

  pub fn read_noncontinuous_word<T: Into<u16>, U: Into<u16>>(&mut self, a: T, b: U) -> u16 {
    (self.read_byte(a) as u16) | (self.read_byte(b) as u16) << 8
  }

  pub fn read_word<T: Into<u16>>(&mut self, address: T) -> u16 {
    let address = address.into();
    self.read_noncontinuous_word(address, address + 1)
  }

  pub fn read_dbg_u8(&mut self, address_start: usize, address_end: usize) -> Vec<u8> {
    if (0x0000..=0x1FFF).contains(&address_start) && (0x0000..=0x1FFF).contains(&address_end) {
      return self.ram[address_start .. address_end].to_vec()
    }
    vec![]
  }

  pub fn clock(&mut self, cpu_cycle: u8) -> u8 {
    let cycle = (cpu_cycle + 1) as u32;

    self.get_mut_apu().step(cycle.into());

    self.nmi.tick();

    self.get_mut_ppu().tick();
    self.get_mut_ppu().tick();
    self.get_mut_ppu().tick();
    0
  }

  pub fn oam_dma_access(&mut self, system_cycles: u32) -> u32 {
    let cpu_dma_cycles = 513 + (system_cycles % 2);
    for idx in 0..=255 {
      let addr = (u16::try_from(self.dma_page).unwrap() << 8) + idx;
      let dma_data = self.read_u8(addr);
      self.get_mut_ppu().get_mut_registers().write_oam_data(dma_data);
    }
    self.dma_transfer = false;
    cpu_dma_cycles
  }
}

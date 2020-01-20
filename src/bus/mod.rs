use std::cell::{RefCell, RefMut};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::cartridge::Cartridge;
use crate::ppu::registers::{PpuCtrlFlags, PpuMaskFlags, Registers, ScrollRegister};
use std::fs::OpenOptions;
use std::io::Write;

pub const MEM_SIZE: usize = 0x0800;

#[cfg(debug_assertions)]
fn init_log_file() {
  let file = OpenOptions::new().write(true).append(false).open("mem.txt").expect("File open error");
  file.set_len(0).unwrap();
}


#[derive(Clone)]
pub struct Bus {
  pub cartridge: Rc<RefCell<Cartridge>>,
  pub ram: [u8; MEM_SIZE],
  controller: Rc<RefCell<[u8; 2]>>,
  controller_state: [u8;2],
  registers: Rc<RefCell<Registers>>
}

impl Bus {
  pub fn new(cartridge: Rc<RefCell<Cartridge>>, registers: Rc<RefCell<Registers>>, controller: Rc<RefCell<[u8; 2]>>) -> Bus {
    let ram = [0u8; MEM_SIZE];
    let controller_state = [0u8; 2];
    init_log_file();

    Bus {
      cartridge,
      ram,
      controller,
      controller_state,
      registers
    }
  }

  #[cfg(debug_assertions)]
  fn log(&self, mode: &str, address: u16, data: u8) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("mem.txt")
        .expect("File append error");

    file
        .write_all(
          format!("{} {} - {}\n", mode, address, data)
              .as_bytes(),
        )
        .expect("File write error");
  }

  fn get_controller(&mut self) -> RefMut<[u8; 2]> {
    self.controller.borrow_mut()
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
      self.log("RAM WRITE", address, data);
      self.ram[usize::try_from(address & 0x07FF).unwrap()] = data;
    } else if (0x2000..=0x3FFF).contains(&address) {
      self.log("PPU WRITE", address, data);
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
      let res = u16::try_from(self.get_mut_cartridge().rom.prg_rom[mapped_addr]).unwrap();
//    println!("A PPU data: {} -> {}", address, res);
      res
    } else if (0x0000..=0x1FFF).contains(&address) {
      let res = u16::try_from(self.ram[usize::try_from(address).unwrap() & 0x07FF]).unwrap();
//    println!("B PPU data: {} -> {}", address, res);
      res
    } else if (0x2000..=0x3FFF).contains(&address) {
      let res = self.get_mut_registers().cpu_read(address & 0x0007, read_only).into();
      self.log("PPU READ", address, u8::try_from(res).unwrap());
//          println!("C PPU data: {} -> {}", address, res);
      res
    } else if (0x4016..=0x4017).contains(&address) {
      self.controller_state[usize::try_from(address & 1).unwrap()] <<= 1;
      let state = self.get_controller()[usize::try_from(address & 0x0001).unwrap()] & 0x80;
      if state > 0x00 { 1 } else { 0 }
//    println!("D PPU data: {} -> {}", address, res);
    } else {
      0
    }
  }
}

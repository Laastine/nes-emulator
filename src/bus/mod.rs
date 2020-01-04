use std::cell::{RefCell, RefMut};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::cartridge::Cartridge;
use crate::mapper::Mapper;
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
    match address {
      0x0000..=0x1FFF => {
        self.ram[usize::try_from(address & 0x07FF).unwrap()] = data;
      }
      0x2000..=0x3FFF => {
        self.write_ppu_registers(address & 0x0007, data)
      }
      0x8000..=0xFFFF => {
        let mapped_addr = usize::try_from(self.get_mut_cartridge().mapper.mapped_write_cpu_u8(address)).unwrap();
        {
          self.get_mut_cartridge().get_prg_rom()[mapped_addr] = data
        };
      }
      _ => panic!("write_u8 address: {} not in range", address),
    }
  }

  fn write_ppu_registers(&mut self, address: u16, data: u8) {
    match address {
      0x00 => {
        self.get_mut_registers().ctrl_flags = PpuCtrlFlags(data);

        let ctrl_flags = self.get_mut_registers().ctrl_flags;
        self.get_mut_registers().tram_addr.set_nametable_x(ctrl_flags.nametable_x());

        self.get_mut_registers().tram_addr.set_nametable_y(ctrl_flags.nametable_y());
      }
      0x01 => {
        self.get_mut_registers().mask_flags = PpuMaskFlags(data);
      },
      0x02 => {},
      0x03 => {},
      0x04 => {},
      0x05 => { // Scroll
        let tram_addr = self.get_mut_registers().tram_addr;
        if self.get_mut_registers().address_latch == 0 { // X
          self.get_mut_registers().fine_x = data & 0x07;

          self.get_mut_registers().tram_addr.set_coarse_x(data >> 3);
          self.get_mut_registers().address_latch = 1;
        } else {                    // Y
          self.get_mut_registers().tram_addr.set_fine_y(data & 0x07);
          self.get_mut_registers().tram_addr.set_coarse_y(data >> 3);
          self.get_mut_registers().address_latch = 0;
        }
      },
      0x06 => { // PPU address
        let tram_addr = self.get_mut_registers().tram_addr.bits();
        if self.get_mut_registers().address_latch == 0 {
          self.get_mut_registers().tram_addr = ScrollRegister(u16::try_from((data & 0x3F).wrapping_shl(8)).unwrap() | tram_addr & 0x00FF);
          self.get_mut_registers().address_latch = 1;
        } else {
          self.get_mut_registers().tram_addr = ScrollRegister(tram_addr & 0xFF00 | u16::try_from(data).unwrap());
          let new_tram_addr = self.get_mut_registers().tram_addr;
          self.get_mut_registers().vram_addr = new_tram_addr;
          self.get_mut_registers().address_latch = 0;
        }
      },
      0x07 => { // PPU data
        let vram_addr = self.get_mut_registers().vram_addr.bits();
        self.get_mut_registers().ppu_write(vram_addr, data);
        let val = if self.get_mut_registers().ctrl_flags.vram_addr_increment_mode() { 32 } else { 1 };
        self.get_mut_registers().vram_addr = ScrollRegister(vram_addr + val);
      },
      _ => panic!("write_ppu_registers address: {} not in range", address),
    };
  }

  pub fn read_u8(&mut self, address: u16, read_only: bool) -> u16 {
    match address {
      0x0000..=0x1FFF => {
        let idx = usize::try_from(address & 0x07FF).unwrap();
        self.ram[idx].into()
      }
      0x2000..=0x3FFF => {
        self.read_ppu_registers(address & 0x0007, read_only).into()
      },
      0x4016..=0x4017 => {
        let res: u8 = if (self.controller[usize::try_from(address & 0x0001).unwrap()] & 0x80) > 0x00 { 1 } else { 0 };
        self.controller[usize::try_from(address & 0x0001).unwrap()] <<= 1;
        res.into()
      }
      0x8000..=0xFFFF => {
        let mapped_addr = usize::try_from(self.get_mut_cartridge().mapper.mapped_read_cpu_u8(address)).unwrap();
        u16::try_from({
          self.get_mut_cartridge().get_prg_rom()[mapped_addr]
        })
        .unwrap()
      }
      _ => 0x0000,
    }
  }

  fn read_ppu_registers(&mut self, address: u16, read_only: bool) -> u8 {
    if read_only {
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
        0x02 => {   // Status
          let status_flags = self.get_mut_registers().status_flags.bits();
          let res = (status_flags & 0xE0) | (self.get_mut_registers().ppu_data_buffer & 0x1F);
          self.get_mut_registers().status_flags.set_vertical_blank_started(false);
          self.get_mut_registers().address_latch = 0x00;
          res
        }
        0x03 => 0x00,
        0x04 => 0x00,
        0x05 => 0x00,
        0x06 => 0x00,
        0x07 => {   // PPU data
          let mut res = self.get_mut_registers().ppu_data_buffer;
          let vram_addr = self.get_mut_registers().vram_addr;
          let ppu_val = self.get_mut_registers().ppu_read(vram_addr.bits());
          self.get_mut_registers().ppu_data_buffer = u8::try_from(ppu_val).unwrap();

          if self.get_mut_registers().vram_addr.bits() > 0x3EFF {
            res = self.get_mut_registers().ppu_data_buffer;
          }
          let vram_addr = self.get_mut_registers().vram_addr;
          let increment_val = if self.get_mut_registers().ctrl_flags.vram_addr_increment_mode() { 32 } else { 1 };
          self.get_mut_registers().vram_addr = ScrollRegister(vram_addr.bits() + increment_val);
          res
        }
        _ => panic!("read_ppu_u8 address: {} not in range", address),
      }
    }
  }
}

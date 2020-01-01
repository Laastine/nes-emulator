use std::cell::{Ref, RefCell, RefMut};
use std::convert::TryFrom;
use std::rc::Rc;

use bitflags::_core::borrow::Borrow;
use image::{ImageBuffer, Rgb};
use luminance::pixel::Format::RG;
use luminance::pixel::NormRGB8UI;
use luminance::texture::{Dim2, Flat, GenMipmaps, Sampler, Texture};
use luminance_glutin::GlutinSurface;

use crate::bus::Bus;
use crate::nes::constants::{COLORS, RGB, SCREEN_RES_X, SCREEN_RES_Y, SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::cartridge::rom::Mirroring;
use crate::ppu::registers::{PpuCtrlFlags, PpuMaskFlags, PpuStatusFlags, Registers, ScrollRegister};

pub mod registers;

pub struct Ppu {
  bus: Rc<RefCell<Bus>>,
  cycles: u32,
  scan_line: u32,
  pub is_frame_ready: bool,
  image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
  pub texture: Texture<Flat, Dim2, NormRGB8UI>,
  registers: Rc<RefCell<Registers>>,
  table_pattern: [[u8; 4096]; 2],
  table_palette: [u8; 32],
  table_name: [[u8; 1024]; 2],
}

impl Ppu {
  pub fn new(bus: Rc<RefCell<Bus>>, registers: Rc<RefCell<Registers>>, surface: &mut GlutinSurface) -> Ppu {
    let cycles = 0;
    let scan_line = 0;

    let image_buffer = ImageBuffer::from_fn(SCREEN_RES_X, SCREEN_RES_Y, |_, _| image::Rgb([0u8, 0u8, 0u8]));

    let texture: Texture<Flat, Dim2, NormRGB8UI> =
      Texture::new(surface, [SCREEN_RES_X, SCREEN_RES_Y], 0, Sampler::default())
        .expect("Texture create error");

    Ppu {
      bus,
      cycles,
      scan_line,
      is_frame_ready: false,
      image_buffer,
      texture,
      registers,
      table_pattern: [[0; 4096]; 2],
      table_palette: [0; 32],
      table_name: [[0; 1024]; 2],
    }
  }

  pub fn get_mut_registers(&mut self) -> RefMut<Registers> {
    self.registers.borrow_mut()
  }

  pub fn get_mut_bus(&mut self) -> RefMut<Bus> {
    self.bus.borrow_mut()
  }

  fn get_color(&mut self, palette: u8, pixel: u8) -> RGB {
    let addr = {
      let mut bus = self.get_mut_bus();
      u8::try_from(bus.read_u8(u16::try_from(palette.wrapping_shl(2) + pixel).unwrap() + 0x3F00)).unwrap()
    };
    COLORS[usize::try_from(addr & 0x3F).unwrap()]
  }

  fn get_pattern_table(&mut self, index: usize, palette: u8) {
    for tile_y in 0..16 {
      for tile_x in 0..16 {
        let offset = tile_x * 256 * 16;

        for row in 0..8 {
          let mut tile_lsb = {
            let mut bus = self.get_mut_bus();
            u8::try_from(bus.read_u8(u16::try_from(index * 4096 + offset + row).unwrap())).unwrap()
          };
          let mut tile_msb = {
            let mut bus = self.get_mut_bus();
            u8::try_from(bus.read_u8(u16::try_from(index * 4096 + offset + row + 8).unwrap())).unwrap()
          };

          for col in 0..8 {
            let pixel = (tile_lsb & 0x01) + (tile_msb & 0x01);

            tile_lsb >>= 1;
            tile_msb >>= 1;

            let texels = ImageBuffer::from_fn(SCREEN_RES_X, SCREEN_RES_Y, |_x, _y| {
              let x = tile_x * 8 + (7 - col);
              let y = tile_y * 8 + row;
              let rgb = self.get_color(palette, pixel);
              image::Rgb(rgb.color)
            }).into_raw();

            self
              .texture
              .upload_raw(GenMipmaps::No, &texels)
              .expect("Texture update error");
          }
        }
      }
    }
  }

  fn ppu_read(&mut self, address: u16) -> u8 {
    let mut addr = address & 0x3FFF;

    match addr {
      0x0000..=0x1FFF => {
        let first_idx = usize::try_from((addr & 0x1000) >> 12).unwrap();
        let second_idx = usize::try_from(addr & 0x0FFF).unwrap();
        self.table_name[first_idx][second_idx]
      }
      0x2000..=0x3EFF => {
        addr &= 0x0FFF;
        let mirror_mode = self.get_mut_bus().cartridge.rom.rom_header.mirroring;
        let idx = usize::try_from(addr & 0x03FF).unwrap();
        match mirror_mode {
          Mirroring::Vertical => {
            match addr {
              0x0000..=0x03FF => self.table_name[0][idx],
              0x0400..=0x07FF => self.table_name[1][idx],
              0x0800..=0x0BFF => self.table_name[0][idx],
              0x0C00..=0x0FFF => self.table_name[1][idx],
              _ => panic!("Unknown vertical mode table address"),
            }
          }
          Mirroring::Horizontal => {
            match addr {
              0x0000..=0x03FF => self.table_name[0][idx],
              0x0400..=0x07FF => self.table_name[0][idx],
              0x0800..=0x0BFF => self.table_name[1][idx],
              0x0C00..=0x0FFF => self.table_name[1][idx],
              _ => panic!("Unknown horizontal mode table address"),
            }
          }
        }
      }
      0x3F00..=0x3FFF => {
        addr &= 0x001F;
        let idx = match addr {
          0x0010 => 0x0000,
          0x0014 => 0x0004,
          0x0018 => 0x0008,
          0x001C => 0x000C,
          _ => panic!("No palette idx found")
        };
        let mask_flags = self.get_mut_registers().mask_flags.bits();
        self.table_palette[idx] & (if mask_flags > 0x00 { 0x30 } else { 0x3F })
      }
      _ => panic!("Address {} not in range", addr)
    }
  }

  fn ppu_write(&mut self, address: u16, data: u8) {
    let mut addr = address & 0x3FFF;

    match addr {
      0x0000..=0x1FFF => {
        let first_idx = usize::try_from((addr & 0x1000) >> 12).unwrap();
        let second_idx = usize::try_from(addr & 0x0FFF).unwrap();
        self.table_name[first_idx][second_idx] = data;
      }
      0x2000..=0x3EFF => {
        addr &= 0x0FFF;
        let mirror_mode = self.get_mut_bus().cartridge.rom.rom_header.mirroring;
        let idx = usize::try_from(addr & 0x03FF).unwrap();
        match mirror_mode {
          Mirroring::Vertical => {
            match addr {
              0x0000..=0x03FF => self.table_name[0][idx] = data,
              0x0400..=0x07FF => self.table_name[1][idx] = data,
              0x0800..=0x0BFF => self.table_name[0][idx] = data,
              0x0C00..=0x0FFF => self.table_name[1][idx] = data,
              _ => panic!("Unknown vertical mode table address"),
            }
          }
          Mirroring::Horizontal => {
            match addr {
              0x0000..=0x03FF => self.table_name[0][idx] = data,
              0x0400..=0x07FF => self.table_name[0][idx] = data,
              0x0800..=0x0BFF => self.table_name[1][idx] = data,
              0x0C00..=0x0FFF => self.table_name[1][idx] = data,
              _ => panic!("Unknown horizontal mode table address"),
            }
          }
        }
      }
      0x3F00..=0x3FFF => {
        addr &= 0x001F;
        let idx = match addr {
          0x0010 => 0x0000,
          0x0014 => 0x0004,
          0x0018 => 0x0008,
          0x001C => 0x000C,
          _ => panic!("No palette idx found")
        };
        let mask_flags = self.get_mut_registers().mask_flags.bits();
        self.table_palette[idx] = data;
      }
      _ => panic!("Address {} not in range", addr)
    }
  }

  pub fn reset(&mut self) {
    self.scan_line = 0;
    self.cycles = 0;
    self.get_mut_registers().status_flags = PpuStatusFlags::from_bits_truncate(0x00);
    self.get_mut_registers().mask_flags = PpuMaskFlags::from_bits_truncate(0x00);
    self.get_mut_registers().ctrl_flags = PpuCtrlFlags::from_bits_truncate(0x00);
    self.get_mut_registers().vram_addr = ScrollRegister::from_bits_truncate(0x0000);
    self.get_mut_registers().tram_addr = ScrollRegister::from_bits_truncate(0x0000);
  }

  pub fn clock(&mut self) {
    let texels = ImageBuffer::from_fn(SCREEN_WIDTH, SCREEN_HEIGHT, |x, y| {
      if (x * SCREEN_RES_X + y) % 2 == 0 && self.cycles % 3 == 0 {
        image::Rgb(COLORS[2].color)
      } else {
        image::Rgb(COLORS[6].color)
      }
    }).into_raw();

    self
      .texture
      .upload_raw(GenMipmaps::No, &texels)
      .expect("Texture update error");

    self.cycles += 1;

    if self.cycles > 340 {
      self.cycles = 0;
      self.scan_line += 1;
      if self.scan_line > 260 {
        self.scan_line -= 1;
        self.is_frame_ready = true;
      }
    }
  }
}

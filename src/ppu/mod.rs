use std::cell::{RefCell, RefMut};
use std::convert::TryFrom;
use std::rc::Rc;

use image::{ImageBuffer, Luma};
use luminance::pixel::NormRGB8UI;
use luminance::texture::{Dim2, Flat, GenMipmaps, Sampler, Texture};

use crate::bus::Bus;
use crate::nes::constants::{SCREEN_RES_X, SCREEN_RES_Y};
use luminance_glutin::GlutinSurface;

pub struct Ppu {
  bus: Rc<RefCell<Bus>>,
  cycles: u32,
  scan_line: u32,
  pub is_frame_ready: bool,
  image_buffer: ImageBuffer<Luma<u8>, Vec<u8>>,
  pub texture: Texture<Flat, Dim2, NormRGB8UI>,
}

impl Ppu {
  pub fn new(bus: Rc<RefCell<Bus>>, surface: &mut GlutinSurface) -> Ppu {
    let cycles = 0;
    let scan_line = 0;

    let image_buffer = ImageBuffer::from_fn(SCREEN_RES_X, SCREEN_RES_Y, |_, _| image::Luma([0u8]));

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
    }
  }

  pub fn get_mut_bus(&mut self) -> RefMut<Bus> {
    self.bus.borrow_mut()
  }

  pub fn write_cpu_u8(&mut self, address: u16, data: u8) {
    let addr = match address {
      0x00 => 0x00,
      0x01 => 0x00,
      0x02 => 0x00,
      0x03 => 0x00,
      0x04 => 0x00,
      0x05 => 0x00,
      0x06 => 0x00,
      0x07 => 0x00,
      0x08 => 0x00,
      _ => 0x00,
    };
    {
      let mut bus = self.get_mut_bus();
      bus.write_u8(addr, data);
    }
  }

  pub fn read_cpu_u8(&mut self, address: u16) -> u8 {
    let addr = match address {
      0x00 => 0x00,
      0x01 => 0x00,
      0x02 => 0x00,
      0x03 => 0x00,
      0x04 => 0x00,
      0x05 => 0x00,
      0x06 => 0x00,
      0x07 => 0x00,
      0x08 => 0x00,
      _ => 0x00,
    };
    {
      let mut bus = self.get_mut_bus();
      u8::try_from(bus.read_u8(addr)).unwrap()
    }
  }

  pub fn clock(&mut self) {
    let bw_img_tex = ImageBuffer::from_fn(SCREEN_RES_X * 2, SCREEN_RES_Y * 2, |x, _y| {
      if x % 2 == 0 && self.cycles % 3 == 0 {
        image::Luma([68u8])
      } else {
        image::Luma([255u8])
      }
    });

    let texels = bw_img_tex.into_raw();
    self
      .texture
      .upload_raw(GenMipmaps::No, &texels)
      .expect("Upload error");

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

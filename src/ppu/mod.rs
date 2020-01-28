use std::cell::{RefCell, RefMut};
use std::convert::TryFrom;
use std::fs::OpenOptions;
use std::io::Write;
use std::rc::Rc;

use image::{ImageBuffer, Rgb};
use luminance::pixel::NormRGB8UI;
use luminance::texture::{Dim2, Flat, GenMipmaps, Sampler, Texture};
use luminance_glutin::GlutinSurface;

use crate::nes::constants::{Color, COLORS, SCREEN_RES_X, SCREEN_RES_Y};
use crate::ppu::registers::{PpuCtrlFlags, PpuMaskFlags, PpuStatusFlags, Registers, ScrollRegister};

pub mod registers;

pub struct Ppu {
  pub cycles: u32,
  scan_line: i32,
  image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
  pub texture: Texture<Flat, Dim2, NormRGB8UI>,
  registers: Rc<RefCell<Registers>>,
  pub nmi: bool,
  fine_x: u8,
  bg_next_tile_id: u8,
  bg_next_tile_attribute: u8,
  bg_next_tile_lo: u8,
  bg_next_tile_hi: u8,
  bg_shifter_pattern_lo: u16,
  bg_shifter_pattern_hi: u16,
  bg_shifter_attrib_lo: u16,
  bg_shifter_attrib_hi: u16,
  pub frame_ready: bool,
}

impl Ppu {
  pub fn new(registers: Rc<RefCell<Registers>>, surface: &mut GlutinSurface) -> Ppu {
    let image_buffer = ImageBuffer::new(SCREEN_RES_X, SCREEN_RES_Y);

    let texture: Texture<Flat, Dim2, NormRGB8UI> =
      Texture::new(surface, [SCREEN_RES_X, SCREEN_RES_Y], 0, Sampler::default())
        .expect("Texture create error");

    Ppu {
      cycles: 0,
      scan_line: 0,
      image_buffer,
      texture,
      registers,
      nmi: false,
      fine_x: 0,
      bg_next_tile_id: 0,
      bg_next_tile_attribute: 0,
      bg_next_tile_lo: 0,
      bg_next_tile_hi: 0,
      bg_shifter_pattern_lo: 0,
      bg_shifter_pattern_hi: 0,
      bg_shifter_attrib_lo: 0,
      bg_shifter_attrib_hi: 0,
      frame_ready: false,
    }
  }

  pub fn get_mut_registers(&mut self) -> RefMut<Registers> {
    self.registers.borrow_mut()
  }

  fn read_ppu_u8(&mut self, address: u16) -> u8 {
    self.get_mut_registers().ppu_read(address)
  }

  fn get_color(&mut self, palette: u8, pixel: u8) -> Color {
    let idx = self.read_ppu_u8(0x3F00u16.wrapping_add(u16::try_from((palette << 2) + pixel).unwrap()));
    COLORS[usize::try_from(idx).unwrap() & 0x3F]
  }

  pub fn reset(&mut self) {
    self.scan_line = 0;
    self.cycles = 0;
    self.get_mut_registers().status_flags = PpuStatusFlags(0);
    self.get_mut_registers().mask_flags = PpuMaskFlags(0);
    self.get_mut_registers().ctrl_flags = PpuCtrlFlags(0);
    self.get_mut_registers().vram_addr = ScrollRegister(0);
    self.get_mut_registers().tram_addr = ScrollRegister(0);
    self.get_mut_registers().ppu_data_buffer = 0;
    self.get_mut_registers().fine_x = 0;
    self.bg_next_tile_id = 0;
    self.bg_next_tile_attribute = 0;
    self.bg_next_tile_lo = 0;
    self.bg_next_tile_hi = 0;
    self.bg_shifter_pattern_lo = 0;
    self.bg_shifter_pattern_hi = 0;
    self.bg_shifter_attrib_lo = 0;
    self.bg_shifter_attrib_hi = 0;
  }

  fn update_shifters(&mut self) {
    if self.get_mut_registers().mask_flags.show_background() {
      self.bg_shifter_pattern_lo <<= 1;
      self.bg_shifter_pattern_hi <<= 1;

      self.bg_shifter_attrib_lo <<= 1;
      self.bg_shifter_attrib_hi <<= 1;
    }
  }

  fn load_background_shifters(&mut self) {
    self.bg_shifter_pattern_lo = (self.bg_shifter_pattern_lo & 0xFF00) | u16::try_from(self.bg_next_tile_lo).unwrap();
    self.bg_shifter_pattern_hi = (self.bg_shifter_pattern_hi & 0xFF00) | u16::try_from(self.bg_next_tile_hi).unwrap();

    self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00) | (if (self.bg_next_tile_attribute & 1) > 0 { 0xFF } else { 0x00 });
    self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00) | (if (self.bg_next_tile_attribute & 2) > 0 { 0xFF } else { 0x00 });
  }

  fn increment_scroll_x(&mut self) {
    let mask_flags = self.get_mut_registers().mask_flags;

    if mask_flags.show_background() || mask_flags.show_sprites() {
      let vram_addr = self.get_mut_registers().vram_addr;

      if vram_addr.coarse_x() == 31 {
        self.get_mut_registers().vram_addr.set_coarse_x(0);
        self.get_mut_registers().vram_addr.0 ^= 0x0400;
      } else {
        let coarse_x = vram_addr.coarse_x();
        self.get_mut_registers().vram_addr.set_coarse_x(coarse_x + 1);
      }
    }
  }

  fn increment_scroll_y(&mut self) {
    let mask_flags = self.get_mut_registers().mask_flags;

    if mask_flags.show_background() || mask_flags.show_sprites() {
      let vram_addr = self.get_mut_registers().vram_addr;
      let fine_y = vram_addr.fine_y();
      if fine_y < 7 {
        self.get_mut_registers().vram_addr.set_fine_y(fine_y + 1);
      } else {
        self.get_mut_registers().vram_addr.set_fine_y(0);
        let coarse_y = self.get_mut_registers().vram_addr.coarse_y();

        if coarse_y == 29 {
          self.get_mut_registers().vram_addr.set_coarse_y(0);
          self.get_mut_registers().vram_addr.0 ^= 0x0800;
        } else if coarse_y == 31 {
          self.get_mut_registers().vram_addr.set_coarse_y(0);
        } else {
          self.get_mut_registers().vram_addr.set_coarse_y(coarse_y + 1);
        }
      }
    }
  }

  fn transfer_address_x(&mut self) {
    let mask_flags = self.get_mut_registers().mask_flags;
    let tram_addr = self.get_mut_registers().tram_addr;

    if mask_flags.show_background() || mask_flags.show_sprites() {
      self.get_mut_registers().vram_addr.set_nametable_x(tram_addr.nametable_x());
      self.get_mut_registers().vram_addr.set_coarse_x(tram_addr.coarse_x());
    }
  }

  fn transfer_address_y(&mut self) {
    let mask_flags = self.get_mut_registers().mask_flags;
    let tram_addr = self.get_mut_registers().tram_addr;

    if mask_flags.show_background() || mask_flags.show_sprites() {
      self.get_mut_registers().vram_addr.set_fine_y(tram_addr.fine_y());
      self.get_mut_registers().vram_addr.set_nametable_y(tram_addr.nametable_y());
      self.get_mut_registers().vram_addr.set_coarse_y(tram_addr.coarse_y());
    }
  }

  #[allow(dead_code)]
  fn log(&mut self) {
    let mut file = OpenOptions::new()
      .write(true)
      .append(true)
      .open("ppu.txt")
      .expect("File append error");

    let cycle = self.cycles;
    let fine_x = self.fine_x;
    let nmi = if self.nmi { 1 } else { 0 };
    let bg_next_tile_id = self.bg_next_tile_id;
    let bg_next_tile_attrib = self.bg_next_tile_attribute;
    let bg_next_tile_lsb = self.bg_next_tile_lo;
    let bg_next_tile_msb = self.bg_next_tile_hi;
    let bg_shifter_pattern_lo = self.bg_shifter_pattern_lo;
    let bg_shifter_pattern_hi = self.bg_shifter_pattern_hi;
    let bg_shifter_attrib_lo = self.bg_shifter_attrib_lo;
    let bg_shifter_attrib_hi = self.bg_shifter_attrib_hi;
    let scan_line = self.scan_line;
    let reg = self.get_mut_registers();
    file
      .write_all(
        format!(
          "{:?},{},{}, {},{},{},{},{},{},{},{},{} {} -> sta:{:?}, msk:{:?}, ctrl:{:?}, tram:{:?}, vram:{:?}\n",
          cycle,
          scan_line,
          reg.ppu_data_buffer,
          fine_x,
          bg_next_tile_id,
          bg_next_tile_attrib,
          bg_next_tile_lsb,
          bg_next_tile_msb,
          bg_shifter_pattern_lo,
          bg_shifter_pattern_hi,
          bg_shifter_attrib_lo,
          bg_shifter_attrib_hi,
          nmi,
          reg.status_flags.0,
          reg.mask_flags.0,
          reg.ctrl_flags.0,
          reg.tram_addr.0,
          reg.vram_addr.0,
        )
          .as_bytes(),
      )
      .expect("File write error");
  }

  pub fn clock(&mut self) {
    if self.scan_line > -2 && self.scan_line < 240 {
      if self.scan_line == 0 && self.cycles == 0 {
        self.cycles = 1;
      }

      if self.scan_line == -1 && self.cycles == 1 {
        self.get_mut_registers().status_flags.set_vertical_blank(false)
      }

      if (self.cycles > 1 && self.cycles < 258) || (320..=338).contains(&self.cycles) {
        self.update_shifters();

        match (self.cycles - 1) % 8 {
          0x00 => self.load_bg_tile(),
          0x02 => self.fetch_next_bg_tile_attrubute(),
          0x04 => self.fetch_next_bg_tile_lo(),
          0x06 => self.fetch_next_bg_tile_hi(),
          0x07 => self.increment_scroll_x(),
          _ => ()
        }
      }

      if self.cycles == 256 {
        self.increment_scroll_y();
      }

      if self.cycles == 257 {
        self.load_background_shifters();
        self.transfer_address_x();
      }

      if self.cycles == 338 || self.cycles == 340 {
        let vram_addr = self.get_mut_registers().vram_addr;
        self.bg_next_tile_id = self.read_ppu_u8(0x2000 | (vram_addr.0 & 0x0FFF));
      }

      if self.scan_line == -1 && (280..=304).contains(&self.cycles) {
        self.transfer_address_y()
      }
    }

    if self.cycles == 1 && self.scan_line == 241 {
      self.get_mut_registers().status_flags.set_vertical_blank(true);

      if self.get_mut_registers().ctrl_flags.enable_nmi() {
        self.nmi = true;
      }
    }

    self.update_image_buffer();

    self.cycles += 1;
    if self.cycles > 340 {
      self.cycles = 0;
      self.scan_line += 1;

      if self.scan_line > 260 {
        self
          .texture
          .upload_raw(GenMipmaps::No, &self.image_buffer)
          .expect("Texture update error");
        self.scan_line = -1;
        self.frame_ready = true;
      }
    }
  }

  fn update_image_buffer(&mut self) {
    let bit_mux = u16::try_from(0x8000 >> self.fine_x).unwrap();

    let bg_pixel = if self.get_mut_registers().mask_flags.show_background() {
      let p0_pixel = if (self.bg_shifter_pattern_lo & bit_mux) > 0 { 1 } else { 0 };
      let p1_pixel = if (self.bg_shifter_pattern_hi & bit_mux) > 0 { 1 } else { 0 };
      (p1_pixel << 1) | p0_pixel
    } else {
      0x00
    };

    let bg_palette = if self.get_mut_registers().mask_flags.show_background() {
      let p0_palette = if (self.bg_shifter_attrib_lo & bit_mux) > 0 { 1 } else { 0 };
      let p1_palette = if (self.bg_shifter_attrib_hi & bit_mux) > 0 { 1 } else { 0 };
      (p1_palette << 1) | p0_palette
    } else {
      0x00
    };
    let x = self.cycles.wrapping_sub(1);
    let y = if self.scan_line > -1 { u32::try_from(self.scan_line).unwrap() } else { 0xFFF };

    if (0..=255).contains(&x) && (0..=239).contains(&y) {
      let pixel = self.get_color(bg_palette, bg_pixel);
      self.image_buffer.put_pixel(x, 239 - y, Rgb(pixel.val));
    }
  }

  fn fetch_next_bg_tile_hi(&mut self) {
    let ctrl_flags = self.get_mut_registers().ctrl_flags;
    let vram_addr = self.get_mut_registers().vram_addr;

    let addr = (u16::try_from(ctrl_flags.pattern_background()).unwrap() << 12)
      + (u16::try_from(self.bg_next_tile_id).unwrap() << 4)
      + u16::try_from(vram_addr.fine_y()).unwrap() + 8;

    self.bg_next_tile_hi = self.read_ppu_u8(addr);
  }

  fn fetch_next_bg_tile_lo(&mut self) {
    let ctrl_flags = self.get_mut_registers().ctrl_flags;
    let vram_addr = self.get_mut_registers().vram_addr;

    let addr = (u16::try_from(ctrl_flags.pattern_background()).unwrap() << 12)
      + (u16::try_from(self.bg_next_tile_id).unwrap() << 4)
      + u16::try_from(vram_addr.fine_y()).unwrap();

    self.bg_next_tile_lo = self.read_ppu_u8(addr);
  }

  fn fetch_next_bg_tile_attrubute(&mut self) {
    let vram_addr = self.get_mut_registers().vram_addr;

    let nametable_x = u16::try_from(vram_addr.nametable_x()).unwrap();
    let nametable_y = u16::try_from(vram_addr.nametable_y()).unwrap();
    let coarse_x = u16::try_from(vram_addr.coarse_x()).unwrap();
    let coarse_y = u16::try_from(vram_addr.coarse_y()).unwrap();

    self.bg_next_tile_attribute = self.read_ppu_u8(0x23C0
      | (nametable_y << 11)
      | (nametable_x << 10)
      | ((coarse_y >> 2) << 3)
      | (coarse_x >> 2));

    if (vram_addr.coarse_y() & 0x02) > 0 {
      self.bg_next_tile_attribute >>= 4;
    }
    if (vram_addr.coarse_x() & 0x02) > 0 {
      self.bg_next_tile_attribute >>= 2;
    }
    self.bg_next_tile_attribute &= 0x03;
  }

  fn load_bg_tile(&mut self) {
    self.load_background_shifters();
    let vram_addr = self.get_mut_registers().vram_addr;

    self.bg_next_tile_id = self.read_ppu_u8(0x2000 | (vram_addr.0 & 0x0FFF));
  }
}

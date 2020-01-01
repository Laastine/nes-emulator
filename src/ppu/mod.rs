use std::cell::{RefCell, RefMut};
use std::convert::TryFrom;
use std::rc::Rc;

use image::{ImageBuffer, Rgb};
use luminance::pixel::NormRGB8UI;
use luminance::texture::{Dim2, Flat, GenMipmaps, Sampler, Texture};
use luminance_glutin::GlutinSurface;

use crate::bus::Bus;
use crate::nes::constants::{Color, COLORS, SCREEN_RES_X, SCREEN_RES_Y};
use crate::ppu::registers::{PpuCtrlFlags, PpuMaskFlags, PpuStatusFlags, Registers, ScrollRegister};

pub mod registers;

pub struct Ppu {
  bus: Rc<RefCell<Bus>>,
  cycles: u32,
  scan_line: i32,
  pub is_frame_ready: bool,
  image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
  pub texture: Texture<Flat, Dim2, NormRGB8UI>,
  registers: Rc<RefCell<Registers>>,
  nmi: bool,
  fine_x: u8,
  bg_next_tile_id: u8,
  bg_next_tile_attrib: u8,
  bg_next_tile_lsb: u8,
  bg_next_tile_msb: u8,
  bg_shifter_pattern_lo: u16,
  bg_shifter_pattern_hi: u16,
  bg_shifter_attrib_lo: u16,
  bg_shifter_attrib_hi: u16,
}

impl Ppu {
  pub fn new(bus: Rc<RefCell<Bus>>, registers: Rc<RefCell<Registers>>, surface: &mut GlutinSurface) -> Ppu {
    let image_buffer = ImageBuffer::new(SCREEN_RES_X, SCREEN_RES_Y);

    let texture: Texture<Flat, Dim2, NormRGB8UI> =
      Texture::new(surface, [SCREEN_RES_X, SCREEN_RES_Y], 0, Sampler::default())
        .expect("Texture create error");

    Ppu {
      bus,
      cycles: 0,
      scan_line: 0,
      is_frame_ready: false,
      image_buffer,
      texture,
      registers,
      nmi: false,
      fine_x: 0,
      bg_next_tile_id: 0,
      bg_next_tile_attrib: 0,
      bg_next_tile_lsb: 0,
      bg_next_tile_msb: 0,
      bg_shifter_pattern_lo: 0,
      bg_shifter_pattern_hi: 0,
      bg_shifter_attrib_lo: 0,
      bg_shifter_attrib_hi: 0,
    }
  }

  pub fn get_mut_registers(&mut self) -> RefMut<Registers> {
    self.registers.borrow_mut()
  }

  pub fn get_mut_bus(&mut self) -> RefMut<Bus> {
    self.bus.borrow_mut()
  }

  fn read_u8(&mut self, address: u16) -> u16 {
    let mut bus = self.get_mut_bus();
    bus.read_u8(address, false)
  }

  fn get_color(&mut self, palette: u8, pixel: u8) -> Color {
    let addr = u8::try_from(self.read_u8(u16::try_from(palette.wrapping_shl(2) + pixel).unwrap() + 0x3F00)).unwrap();
    COLORS[usize::try_from(addr & 0x3F).unwrap()]
  }

  pub fn reset(&mut self) {
    self.scan_line = 0;
    self.cycles = 0;
    self.get_mut_registers().status_flags = PpuStatusFlags::from_bits_truncate(0x00);
    self.get_mut_registers().mask_flags = PpuMaskFlags::from_bits_truncate(0x00);
    self.get_mut_registers().ctrl_flags = PpuCtrlFlags::from_bits_truncate(0x00);
    self.get_mut_registers().vram_addr = ScrollRegister::from_bits_truncate(0x0000);
    self.get_mut_registers().tram_addr = ScrollRegister::from_bits_truncate(0x0000);
    self.bg_next_tile_id = 0;
    self.bg_next_tile_attrib = 0;
    self.bg_next_tile_lsb = 0;
    self.bg_next_tile_msb = 0;
    self.bg_shifter_pattern_lo = 0;
    self.bg_shifter_pattern_hi = 0;
    self.bg_shifter_attrib_lo = 0;
    self.bg_shifter_attrib_hi = 0;
  }

  fn update_shifters(&mut self) {
    self.bg_shifter_pattern_lo <<= 1;
    self.bg_shifter_pattern_hi <<= 1;

    self.bg_shifter_attrib_lo <<= 1;
    self.bg_shifter_attrib_hi <<= 1;
  }

  fn load_background_shifters(&mut self) {
    self.bg_shifter_pattern_lo = (self.bg_shifter_pattern_lo & 0xFF00) | u16::try_from(self.bg_next_tile_lsb).unwrap();
    self.bg_shifter_pattern_hi = (self.bg_shifter_pattern_hi & 0xFF00) | u16::try_from(self.bg_next_tile_msb).unwrap();

    self.bg_shifter_attrib_lo = self.bg_shifter_attrib_lo & 0xFF00 | (if self.bg_next_tile_attrib & 0b01 > 0x00 { 0xFF } else { 0x00 });
    self.bg_shifter_attrib_hi = self.bg_shifter_attrib_hi & 0xFF00 | (if self.bg_next_tile_attrib & 0b10 > 0x00 { 0xFF } else { 0x00 });
  }

  fn increment_scroll_x(&mut self) {
    let mask_flags = self.get_mut_registers().mask_flags;
    let show_background = mask_flags.contains(PpuMaskFlags::SHOW_BACKGROUND);
    let show_sprites = mask_flags.contains(PpuMaskFlags::SHOW_SPRITES);

    let vram_addr = self.get_mut_registers().vram_addr;

    if show_background || show_sprites {
      if vram_addr.contains(ScrollRegister::COARSE_X) {
        self.get_mut_registers().vram_addr.set(ScrollRegister::COARSE_X, false);
        let new_val = !self.get_mut_registers().vram_addr.contains(ScrollRegister::NAMETABLE_X);
        self.get_mut_registers().vram_addr.set(ScrollRegister::NAMETABLE_X, new_val);
      } else {
        let vram_addr = self.get_mut_registers().vram_addr.bits();
        self.get_mut_registers().vram_addr.insert(ScrollRegister::from_bits(vram_addr + 1).unwrap());
      }
    }
  }

  fn increment_scroll_y(&mut self) {
    let mask_flags = self.get_mut_registers().mask_flags;
    let show_background = mask_flags.contains(PpuMaskFlags::SHOW_BACKGROUND);
    let show_sprites = mask_flags.contains(PpuMaskFlags::SHOW_SPRITES);

    if show_background || show_sprites {
      let vram_addr = self.get_mut_registers().vram_addr;


    }
  }

  fn transfer_address_x(&mut self) {
    let mask_flags = self.get_mut_registers().mask_flags;
    let show_background = mask_flags.contains(PpuMaskFlags::SHOW_BACKGROUND);
    let show_sprites = mask_flags.contains(PpuMaskFlags::SHOW_SPRITES);

    let vram_addr = self.get_mut_registers().vram_addr;

    if show_background || show_sprites {

    }
  }

  fn transfer_address_y(&mut self) {
    unimplemented!()
  }

  pub fn clock(&mut self) {
    if self.scan_line > -2 && self.scan_line < 240 {
      if self.scan_line == 0 && self.cycles == 0 {
        self.cycles = 1;
      }

      if self.scan_line == -1 && self.cycles == 1 {
        self.get_mut_registers().status_flags.set(PpuStatusFlags::VERTICAL_BLANK_STARTED, false)
      }

      if self.cycles > 1 && self.cycles < 258 || (self.cycles > 320 && self.cycles < 338) {
        self.update_shifters();

        match self.cycles - 1 % 8 {
          0x00 => {
            self.load_background_shifters();
            let vram_addr = self.get_mut_registers().vram_addr;

            let new_tile_id = self.get_mut_registers().ppu_read(0x2000 | (vram_addr.bits() & 0x0FFF));
            self.bg_next_tile_id = new_tile_id;
          }
          0x02 => {
            let vram_addr = self.get_mut_registers().vram_addr;

            let nametable_x: u16 = if vram_addr.contains(ScrollRegister::NAMETABLE_X) { 1 } else { 0 };
            let nametable_y: u16 = if vram_addr.contains(ScrollRegister::NAMETABLE_Y) { 1 } else { 0 };
            let coarse_x: u16 = if vram_addr.contains(ScrollRegister::COARSE_X) { 1 } else { 0 };
            let coarse_y: u16 = if vram_addr.contains(ScrollRegister::COARSE_Y) { 1 } else { 0 };

            let new_tile_id = self.get_mut_registers().ppu_read(0x23C0 | (vram_addr.bits() & 0x0FFF)
              | (nametable_y << 11)
              | (nametable_x << 10)
              | ((coarse_y >> 2) << 3)
              | (coarse_x >> 2));
            self.bg_next_tile_id = new_tile_id;

            if (coarse_y & 0x02) > 0x00 {
              self.bg_next_tile_attrib >>= 4;
            }
            if (coarse_x & 0x02) > 0x00 {
              self.bg_next_tile_attrib >>= 2;
            }
            self.bg_next_tile_attrib &= 0x03;
          }
          0x04 => {
            let ctrl_flags = self.get_mut_registers().ctrl_flags;
            let vram_addr = self.get_mut_registers().vram_addr;
            let pattern_background: u8 = if ctrl_flags.contains(PpuCtrlFlags::PATTERN_BACKGROUND_TABLE_ADDR) { 1 } else { 0 };
            let fine_y = if vram_addr.contains(ScrollRegister::FINE_Y) { 1 } else { 0 };

            let addr = u16::try_from(pattern_background.wrapping_shl(12)).unwrap() + u16::try_from(self.bg_next_tile_id).unwrap() + fine_y + 8;

            let new_tile_lsb = self.get_mut_registers().ppu_read(addr);
            self.bg_next_tile_lsb = new_tile_lsb;
          }
          0x06 => {
            let ctrl_flags = self.get_mut_registers().ctrl_flags;
            let vram_addr = self.get_mut_registers().vram_addr;
            let pattern_background: u8 = if ctrl_flags.contains(PpuCtrlFlags::PATTERN_BACKGROUND_TABLE_ADDR) { 1 } else { 0 };
            let fine_y = if vram_addr.contains(ScrollRegister::FINE_Y) { 1 } else { 0 };

            let addr = u16::try_from(pattern_background.wrapping_shl(12)).unwrap() + u16::try_from(self.bg_next_tile_id).unwrap() + fine_y + 8;
            let new_tile_msb = self.get_mut_registers().ppu_read(addr);
            self.bg_next_tile_msb = new_tile_msb;
          }
          0x07 => {
            self.increment_scroll_x();
          }
          _ => ()
        }
      }

      if self.cycles == 256 {
        self.increment_scroll_y();
      }

      if self.cycles == 256 {
        self.load_background_shifters();
        self.transfer_address_x();
      }

      if self.cycles == 338 || self.cycles == 340 {
        let vram_addr = self.get_mut_registers().vram_addr;
        let new_tile_id = self.get_mut_registers().ppu_read(0x2000 | vram_addr.bits() & 0x0FFF);
        self.bg_next_tile_id = new_tile_id;
      }

      if self.scan_line == -1 && (280..=304).contains(&self.cycles) {
        self.transfer_address_y()
      }
    }

    if self.scan_line == 240 {
      // do nothing
    }

    if self.cycles == 1 && (241..=260).contains(&self.scan_line) {
      self.get_mut_registers().status_flags.set(PpuStatusFlags::VERTICAL_BLANK_STARTED, true);

      if self.get_mut_registers().ctrl_flags.contains(PpuCtrlFlags::ENABLE_NMI) {
        self.nmi = true;
      }
    }

    let mut bg_pixel: u8 = 0x00;
    let mut bg_palette: u8 = 0x00;

    if self.get_mut_registers().mask_flags.contains(PpuMaskFlags::SHOW_BACKGROUND) {
      let bit_mux = u16::try_from((0x8000_i32).wrapping_shr(self.fine_x.into())).unwrap();

      let p0_pixel = if (self.bg_shifter_pattern_lo & bit_mux) > 0x00 { 1 } else { 0 };
      let p1_pixel = if (self.bg_shifter_pattern_hi & bit_mux) > 0x00 { 1 } else { 0 };

      bg_pixel = (p1_pixel << 1) | p0_pixel;

      let p0_palette = if (self.bg_shifter_attrib_lo & bit_mux) > 0x00 { 1 } else { 0 };
      let p1_palette = if (self.bg_shifter_attrib_hi & bit_mux) > 0x00 { 1 } else { 0 };

      bg_palette = (p0_palette << 1) | p1_palette;
    }

//    dbg!(bg_palette, bg_pixel);
    let pixel = self.get_color(bg_palette, bg_pixel);

    let x = self.cycles - 1;
    let y = u32::try_from(self.scan_line).unwrap();

//    dbg!(self.scan_line, y);

    self.image_buffer.put_pixel(x, y, Rgb(pixel.val));

    self.cycles += 1;
    if self.cycles > 340 {
      self.cycles = 0;
      dbg!("increment scan_line");
      self.scan_line += 1;

      if self.scan_line > 260 {
        self.scan_line = -1;
        self
          .texture
          .upload_raw(GenMipmaps::No, &self.image_buffer)
          .expect("Texture update error");
        self.is_frame_ready = true;
      }
    }
  }
}

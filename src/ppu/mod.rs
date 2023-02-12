use std::cell::{Ref, RefCell, RefMut};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::nes::constants::{Color, COLORS};
use crate::nes::OffScreenBuffer;
use crate::ppu::oam_sprite::Sprite;
use crate::ppu::registers::{get_nth_bit, Registers};

pub mod registers;
mod oam_sprite;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum PpuState {
  Render,
  NonMaskableInterrupt,
  NoOp,
  Scanline,
}

pub struct Ppu {
  pub cycles: usize,
  scan_line: usize,
  registers: Rc<RefCell<Registers>>,
  pub nmi: bool,
  nametable_entry: u8,
  bg_next_tile_attribute: u8,
  bg_next_tile_lo: u8,
  bg_next_tile_hi: u8,
  bg_shifter_lo: u16,
  bg_shifter_hi: u16,
  bg_attribute_latch_lo: u8,
  bg_attribute_latch_hi: u8,
  curr_address: u16,
  attribute_shift_lo: u8,
  attribute_shift_hi: u8,
  pub is_frame_ready: bool,
  primary_oam: Vec<Sprite>,
  secondary_oam: Vec<Sprite>,
  pub is_even_frame: bool,
  off_screen_pixels: Rc<RefCell<OffScreenBuffer>>,
}

impl Ppu {
  pub fn new(registers: Rc<RefCell<Registers>>, off_screen_pixels: Rc<RefCell<OffScreenBuffer>>) -> Ppu {
    Ppu {
      cycles: 0,
      scan_line: 0,
      registers,
      nmi: false,
      nametable_entry: 0,
      bg_next_tile_attribute: 0,
      bg_next_tile_lo: 0,
      bg_next_tile_hi: 0,
      bg_shifter_lo: 0,
      bg_shifter_hi: 0,
      bg_attribute_latch_lo: 0,
      bg_attribute_latch_hi: 0,
      curr_address: 0,
      attribute_shift_lo: 0,
      attribute_shift_hi: 0,
      primary_oam: Vec::with_capacity(8),
      secondary_oam: Vec::with_capacity(8),
      is_frame_ready: false,
      is_even_frame: true,
      off_screen_pixels,
    }
  }

  #[inline]
  pub fn get_mut_registers(&mut self) -> RefMut<Registers> {
    self.registers.borrow_mut()
  }

  #[inline]
  fn get_mut_off_screen_pixels(&mut self) -> RefMut<OffScreenBuffer> {
    self.off_screen_pixels.borrow_mut()
  }

  #[inline]
  pub fn get_registers(&self) -> Ref<Registers> {
    self.registers.borrow()
  }

  #[inline]
  pub fn read_ppu_u8(&mut self, address: u16) -> u8 {
    self.get_mut_registers().ppu_read_reg(address)
  }

  #[inline]
  pub fn write_ppu_u8(&mut self, address: u16, data: u8) {
    self.get_mut_registers().bus_write_ppu_reg(address, data);
  }

  fn get_pixel_color(&mut self, pixel: u8) -> Color {
    let palette: u16 = if self.get_registers().mask_flags.is_rendering() {
      pixel
    } else {
      0
    }.into();
    let idx = self.read_ppu_u8(0x3F00 + palette);
    COLORS[usize::try_from(idx).unwrap()]
  }

  pub fn reset(&mut self) {
    self.scan_line = 0;
    self.cycles = 0;
    self.nametable_entry = 0;
    self.bg_next_tile_attribute = 0;
    self.bg_next_tile_lo = 0;
    self.bg_next_tile_hi = 0;
    self.bg_shifter_lo = 0;
    self.bg_shifter_hi = 0;
    self.bg_attribute_latch_lo = 0;
    self.bg_attribute_latch_hi = 0;
    self.attribute_shift_lo = 0;
    self.attribute_shift_hi = 0;
    self.primary_oam.clear();
    self.secondary_oam.clear();
    self.is_frame_ready = false;
    self.is_even_frame = true;
    self.get_mut_registers().reset();
  }

  fn update_shifters(&mut self) {
    self.bg_shifter_lo <<= 1;
    self.bg_shifter_hi <<= 1;

    self.attribute_shift_lo = self.attribute_shift_lo << 1 | self.bg_attribute_latch_lo;
    self.attribute_shift_hi = self.attribute_shift_hi << 1 | self.bg_attribute_latch_hi;
  }

  fn load_background_shifters(&mut self) {
    self.bg_shifter_lo = (self.bg_shifter_lo & 0xFF00) | self.bg_next_tile_lo as u16;
    self.bg_shifter_hi = (self.bg_shifter_hi & 0xFF00) | self.bg_next_tile_hi as u16;

    self.bg_attribute_latch_lo = self.bg_next_tile_attribute & 1;
    self.bg_attribute_latch_hi = (self.bg_next_tile_attribute & 2) >> 1;
  }

  fn increment_scroll_x(&mut self) {
    if self.get_registers().mask_flags.is_rendering() {
      if self.get_registers().vram_addr.coarse_x() == 31 {
        self.get_mut_registers().vram_addr.set_coarse_x(0);
        self.get_mut_registers().vram_addr.0 ^= 0x0400;
      } else {
        let coarse_x = self.get_registers().vram_addr.coarse_x();
        self.get_mut_registers().vram_addr.set_coarse_x(coarse_x + 1);
      }
    }
  }

  fn increment_scroll_y(&mut self) {
    if self.get_registers().mask_flags.is_rendering() {
      let fine_y = self.get_registers().vram_addr.fine_y();
      if fine_y < 7 {
        self.get_mut_registers().vram_addr.set_fine_y(fine_y + 1);
      } else {
        self.get_mut_registers().vram_addr.set_fine_y(0);
        let coarse_y = self.get_registers().vram_addr.coarse_y();

        if coarse_y == 29 {
          self.get_mut_registers().vram_addr.set_coarse_y(0);
          let nametable_y = self.get_mut_registers().vram_addr.nametable_y();
          self.get_mut_registers().vram_addr.set_nametable_y(!nametable_y);
        } else if coarse_y == 31 {
          self.get_mut_registers().vram_addr.set_coarse_y(0);
        } else {
          self.get_mut_registers().vram_addr.set_coarse_y(coarse_y + 1);
        }
      }
    }
  }

  fn transfer_address_x(&mut self) {
    if self.get_registers().mask_flags.is_rendering() {
      let tram_addr = self.get_registers().tram_addr;
      self.get_mut_registers().vram_addr.set_nametable_x(tram_addr.nametable_x());
      self.get_mut_registers().vram_addr.set_coarse_x(tram_addr.coarse_x());
    }
  }

  fn transfer_address_y(&mut self) {
    if self.get_registers().mask_flags.is_rendering() {
      let tram_addr = self.get_registers().tram_addr;
      self.get_mut_registers().vram_addr.set_fine_y(tram_addr.fine_y());
      self.get_mut_registers().vram_addr.set_nametable_y(tram_addr.nametable_y());
      self.get_mut_registers().vram_addr.set_coarse_y(tram_addr.coarse_y());
    }
  }

  pub fn tick(&mut self) -> PpuState {
    let mut state = PpuState::NoOp;
    match self.scan_line {
      (0..=239) => {
        self.process_sprites(false);
        self.update_image_buffer();
        self.process_background(false);
      }
      261 => {
        self.process_sprites(true);
        self.update_image_buffer();
        self.process_background(true);
      }
      240 => {
        if self.cycles == 0 {
          self.is_frame_ready = true;
          state = PpuState::Render
        }
      }
      241 => {
        if self.cycles == 1 && !self.get_registers().vblank_suppress {
          self.get_mut_registers().status_flags.set_vertical_blank(true);

          if self.get_registers().ctrl_flags.enable_nmi() {
            self.nmi = true;
            state = PpuState::NonMaskableInterrupt
          } else {
            self.nmi = false;
            state = PpuState::NoOp
          }
        }
      }
      _ => ()
    }

    if self.get_mut_registers().status_flags.vertical_blank() && self.get_mut_registers().force_nmi && !self.get_mut_registers().vblank_suppress {
        if let PpuState::NoOp = state {
          state = PpuState::NonMaskableInterrupt;
        } else {
          panic!("Invalid state");
        }
      };

    self.get_mut_registers().force_nmi = false;
    self.get_mut_registers().vblank_suppress = false;

    self.cycles += 1;
    let mask_flags = self.get_registers().mask_flags;
    if mask_flags.is_rendering() && self.cycles == 260 && self.scan_line < 240 {
      self.get_mut_registers().get_mut_cartridge().mapper.signal_scanline();
    }

    if self.cycles > 340 {
      self.cycles = 0;
      self.scan_line += 1;

      if self.scan_line > 261 {
        self.scan_line = 0;
        self.is_even_frame = !self.is_even_frame;
      }
    }

    state
  }

  fn process_background(&mut self, is_pre_render: bool) {
    if (2..=255).contains(&self.cycles) || (322..=337).contains(&self.cycles) {
      match self.cycles % 8 {
        0x01 => {
          let vram_addr = self.get_registers().vram_addr;
          self.curr_address = 0x2000 | (vram_addr.0 & 0x0FFF);
          self.load_background_shifters();
        }
        0x02 => {
          self.nametable_entry = self.read_ppu_u8(self.curr_address);
        }
        0x03 => {
          self.curr_address = self.fetch_next_bg_tile_attribute();
        }
        0x04 => {
          self.bg_next_tile_attribute = self.read_ppu_u8(self.curr_address);
          if (self.get_registers().vram_addr.coarse_y() & 0x02) > 0 {
            self.bg_next_tile_attribute >>= 4;
          }
          if (self.get_registers().vram_addr.coarse_x() & 0x02) > 0 {
            self.bg_next_tile_attribute >>= 2;
          }
        }
        0x05 => {
          let vram_addr = self.get_registers().vram_addr;
          let ctrl_flags = self.get_registers().ctrl_flags;
          self.curr_address = ctrl_flags.get_pattern_background()
            + ((0x10 * u16::try_from(self.nametable_entry).unwrap()) | u16::try_from(vram_addr.fine_y()).unwrap());
        }
        0x06 => {
          self.bg_next_tile_lo = self.read_ppu_u8(self.curr_address);
        }
        0x07 => {
          self.curr_address += 8;
        }
        0x00 => {
          self.bg_next_tile_hi = self.read_ppu_u8(self.curr_address);
          if self.get_registers().mask_flags.is_rendering() {
            self.increment_scroll_x();
          }
        }
        _ => panic!("Invalid cycle, modulo operation error"),
      }
    }

    if self.cycles == 256 {
      self.bg_next_tile_hi = self.read_ppu_u8(self.curr_address);
      self.increment_scroll_y();
    }

    if self.cycles == 257 {
      self.load_background_shifters();
      self.transfer_address_x();
    }

    if (280..=304).contains(&self.cycles) && is_pre_render {
      self.transfer_address_y()
    }

    if self.cycles == 1 {
      let vram_addr = self.get_registers().vram_addr;
      self.curr_address = 0x2000 | (vram_addr.0 & 0x0FFF);
      if is_pre_render {
        self.get_mut_registers().status_flags.set_vertical_blank(false);
      }
    }

    if self.cycles == 321 || self.cycles == 339 {
      let vram_addr = self.get_registers().vram_addr;
      self.curr_address = 0x2000 | (vram_addr.0 & 0x0FFF);
    }

    if self.cycles == 338 {
      self.nametable_entry = self.read_ppu_u8(self.curr_address);
    }

    if self.cycles == 340 {
      self.nametable_entry = self.read_ppu_u8(self.curr_address);

      if is_pre_render && self.get_registers().mask_flags.is_rendering() && !self.is_even_frame {
        self.cycles += 1;
      }
    }
  }

  fn process_sprites(&mut self, is_pre_render: bool) {
    match self.cycles {
      1 => {
        self.secondary_oam.clear();
        if is_pre_render {
          self.get_mut_registers().status_flags.set_sprite_overflow(false);
          self.get_mut_registers().status_flags.set_sprite_zero_hit(false);
        }
      }
      257 => self.evaluate_sprites(),
      321 => self.load_sprites(),
      _ => ()
    }
  }

  fn evaluate_sprites(&mut self) {
    self.secondary_oam.clear();
    for idx in 0..=63 {
      let address = idx * 4;
      let sprite = Sprite::new(idx, &self.get_registers().oam_ram[address..(address + 4)]);

      let sprite_size = usize::try_from(self.get_registers().ctrl_flags.get_sprite_size()).unwrap();
      let scan_line = self.scan_line;
      let sprite_y = usize::try_from(sprite.y).unwrap();

      if scan_line >= sprite_y && scan_line < (sprite_y + sprite_size) {
        if self.secondary_oam.len() == 8 {
          self.get_mut_registers().status_flags.set_sprite_overflow(true);
          break;
        }
        self.secondary_oam.push(sprite);
      }
    }
  }

  fn load_sprites(&mut self) {
    let mut sprites = self.secondary_oam.clone();
    for sprite in sprites.iter_mut() {
      let scan_line = self.scan_line;
      let tile_address = sprite.tile_address(self.get_registers().ctrl_flags, scan_line);
      sprite.data_lo = self.get_registers().ppu_read_reg(tile_address);
      sprite.data_hi = self.get_registers().ppu_read_reg(tile_address + 8);
    }
    self.primary_oam = sprites;
  }

  fn render_sprite_pixel(&mut self, x: usize) -> (u8, bool, bool) {
    let mut color = 0;
    let mut is_behind = false;
    let mut possible_zero_hit = false;

    if self.get_registers().mask_flags.is_rendering_sprites(x) {
      for sprite in self.primary_oam.iter().rev() {
        let sprite_color_idx = sprite.color_index(x);

        if sprite_color_idx > 0 {
          possible_zero_hit = sprite.oam_index == 0 && x != 0xFF;
          color = 0b1_00_00 | sprite.attributes.palette() << 2 | sprite_color_idx;
          is_behind = sprite.attributes.is_behind_background();
        }
      }
    }

    (color, is_behind, possible_zero_hit)
  }

  fn render_background_pixel(&mut self, x: usize) -> u8 {
    let mut res = 0;
    if self.get_registers().mask_flags.is_rendering_background(x) {
      let nth = 15 - self.get_registers().fine_x;
      res = get_nth_bit(self.bg_shifter_hi, nth) << 1 | get_nth_bit(self.bg_shifter_lo, nth);

      if res != 0 {
        let nth = 7 - self.get_registers().fine_x;
        res |= (get_nth_bit(self.attribute_shift_hi, nth) << 1 | get_nth_bit(self.attribute_shift_lo, nth)) << 2;
      }
    }
    res
  }

  fn update_image_buffer(&mut self) {
    if (2..=257).contains(&self.cycles) || (322..=337).contains(&self.cycles) {
      let x = self.cycles - 2;
      let y = self.scan_line;

      if let Some(color) = self.get_screen_pixel(x, y) {
        let pixel = self.get_pixel_color(color).to_value();
        self.get_mut_off_screen_pixels()[(239 - y) * 256 + x] = pixel;
      }
      self.update_shifters();
    }
  }

  fn get_screen_pixel(&mut self, x: usize, y: usize) -> Option<u8> {
    if x < 256 && y < 240 {
      let bg_pixel = self.render_background_pixel(x);
      let (sprite_pixel, sprite_behind, possible_zero_hit) = self.render_sprite_pixel(x);

      if possible_zero_hit && bg_pixel != 0 {
        self.get_mut_registers().status_flags.set_sprite_zero_hit(true);
      }

      let colors = if !sprite_behind {
        [sprite_pixel, bg_pixel]
      } else {
        [bg_pixel, sprite_pixel]
      };
      let color = if colors[0] > 0 { colors[0] } else { colors[1] };

      return Some(color)
    }
    None
  }

  fn fetch_next_bg_tile_attribute(&mut self) -> u16 {
    let vram_addr = self.get_registers().vram_addr;

    let nametable_x = u16::try_from(vram_addr.nametable_x()).unwrap();
    let nametable_y = u16::try_from(vram_addr.nametable_y()).unwrap();
    let coarse_x = u16::try_from(vram_addr.coarse_x()).unwrap();
    let coarse_y = u16::try_from(vram_addr.coarse_y()).unwrap();

    0x23C0 | (nametable_y << 11) | (nametable_x << 10) | ((coarse_y >> 2) << 3) | (coarse_x >> 2)
  }
}

use std::convert::TryFrom;

use crate::ppu::registers::{get_nth_bit, PpuCtrlFlags};

bitfield! {
  #[derive(Copy, Clone, PartialEq)]
  pub struct SpriteAttributes(u8); impl Debug;
  pub palette,              _: 1, 0;
  pub is_behind_background, _:    5;
  pub flip_x,               _:    6;
  pub flip_y,               _:    7;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SpriteTileIndex(u8);

#[derive(Debug, Copy, Clone)]
pub struct Sprite {
  pub y: u8,
  pub index: SpriteTileIndex,
  pub attributes: SpriteAttributes,
  pub x: u8,
  pub data_lo: u8,
  pub data_hi: u8,
  pub oam_index: usize,
}

impl Sprite {
  pub fn new(oam_index: usize, bytes: &[u8]) -> Sprite {
    Sprite {
      y: bytes[0],
      index: SpriteTileIndex(bytes[1]),
      attributes: SpriteAttributes(bytes[2]),
      x: bytes[3],
      data_lo: 0,
      data_hi: 0,
      oam_index,
    }
  }

  pub fn tile_address(&mut self, control_flags: PpuCtrlFlags, scan_line: usize) -> u16 {
    let tile_address = if control_flags.sprite_size() {
      0x1000 * u16::try_from(self.index.0 & 1).unwrap() + 0x10 * u16::try_from(self.index.0 & !1).unwrap()
    } else {
      control_flags.get_sprite_tile_base() + 0x10 * u16::try_from(self.index.0).unwrap()
    };

    let sprite_size = control_flags.get_sprite_size();
    let mut y_offset = ((scan_line - self.y as usize) as u16 % sprite_size as u16) as u16;

    if self.attributes.flip_y() {
      y_offset = control_flags.get_sprite_size() - 1 - y_offset;
    }

    tile_address + y_offset + if y_offset < 8 { 0 } else { 8 }
  }

  pub fn color_index(&self, x: usize) -> u8 {
    let mut sprite_x = x.wrapping_sub(self.x as usize) as u16;
    if sprite_x < 8 {
      if self.attributes.flip_x() {
        sprite_x = 7 - sprite_x;
      }
      get_nth_bit(self.data_hi, 7 - sprite_x) << 1 | get_nth_bit(self.data_lo, 7 - sprite_x)
    } else {
      0
    }
  }
}


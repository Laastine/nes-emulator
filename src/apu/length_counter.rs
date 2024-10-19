
const LENGTH_TABLE: [u8; 32] = [
  0xA, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0xA, 0x0E, 0x0C, 0x1A, 0xE,
  0xC, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E
];


pub struct LengthCounter {
  pub is_enabled: bool,
  is_halt: bool,
  frame_counter: u8,
  is_pending: Option<bool>,
  is_pending_reg: Option<u8>,
}

impl LengthCounter {
  pub fn new() -> LengthCounter {
    LengthCounter {
      is_enabled: false,
      is_halt: false,
      frame_counter: 0,
      is_pending: None,
      is_pending_reg: None,
    }
  }

  pub fn write_register(&mut self, val: u8) {
    self.is_pending_reg = Some(val);
  }

  pub fn set_halted(&mut self, val: bool) {
    self.is_pending = Some(val);
  }

  pub fn set_enabled(&mut self, val: bool) {
    self.is_enabled = val;
    if !val {
      self.frame_counter = 0;
    }
  }

  pub fn step(&mut self) {
    if self.is_pending.is_some() {
      if self.frame_counter == 0 {
        return;
      } else {
        self.is_pending = None;
      }
    }
    if self.is_enabled && !self.is_halt && self.frame_counter > 0 {
      self.frame_counter = self.frame_counter.wrapping_sub(1);
    }
  }

  pub fn update_pending(&mut self) {
    if let Some(v) = self.is_pending {
      self.is_halt = v;
      self.is_pending = None;
    }

    if let Some(val) = self.is_pending_reg {
      if self.is_enabled {
        self.frame_counter = LENGTH_TABLE[usize::from(val >> 3)];
      }
      self.is_pending_reg = None;
    }
  }

  pub fn active(&self) -> bool {
    self.is_enabled && self.frame_counter > 0
  }

  pub fn playing(&self) -> bool {
    self.frame_counter > 0
  }
}

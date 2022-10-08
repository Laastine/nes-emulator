bitfield! {
  #[derive(Copy, Clone, Eq, PartialEq)]
  pub struct EnvelopeCtrl(u8); impl Debug;
  pub constant_volume,    _: 3, 0;
  pub decay_level,        _: 3, 0;
  pub constant_flag,      _: 4, 4;
  pub loop_flag,          _: 5, 5;
}

pub struct Envelope {
  ctrl: EnvelopeCtrl,
  length_counter: u8,
  volume_level: u8,
  is_start: bool,
}

impl Envelope {
  pub fn new() -> Envelope {
    Envelope {
      ctrl: EnvelopeCtrl(0),
      length_counter: 0,
      volume_level: 0,
      is_start: false,
    }
  }

  pub fn step(&mut self) {
    if self.is_start {
      self.is_start = false;
      self.set_volume_level(0x0F);
    } else if self.length_counter == 0 {
      if self.volume_level > 0 {
        self.set_volume_level(self.volume_level - 1)
      } else if self.ctrl.loop_flag() > 0 {
        self.set_volume_level(0x0F);
      }
    }
  }

  pub fn write_reg(&mut self, data: u8) {
    self.ctrl = EnvelopeCtrl(data);
  }

  pub fn start(&mut self) {
    self.is_start = true;
  }

  pub fn get_volume_level(&self) -> u8 {
    if self.ctrl.constant_flag() > 0 {
      self.ctrl.constant_volume()
    } else {
      self.volume_level
    }
  }

  fn set_volume_level(&mut self, volume_val: u8) {
    self.volume_level = volume_val & 0x0F;
    self.length_counter = self.ctrl.decay_level();
  }
}

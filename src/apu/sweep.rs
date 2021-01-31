use crate::apu::sequencer::Sequencer;

#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
  OnesComplement = 1,
  TwosComplement = 0,
}

impl Mode {
  pub fn value(&self) -> u16 {
    match *self {
      Mode::OnesComplement => 0x01,
      Mode::TwosComplement => 0x00,
    }
  }
}

pub struct Sweep {
  is_enabled: bool,
  is_reload: bool,
  shift_amount: u8,
  is_negate: bool,
  negate_mode: Mode,
  current_period: u8,
  frame_counter: u8,
}

impl Sweep {
  pub fn new(negate_mode: Mode) -> Sweep {
    Sweep {
      is_enabled: false,
      is_reload: false,
      shift_amount: 0,
      is_negate: false,
      negate_mode,
      current_period: 0,
      frame_counter: 0,
    }
  }

  pub fn step(&mut self, sequencer: &mut Sequencer) {
    if self.current_period == 0 && self.is_enabled && self.shift_amount > 0 && sequencer.period > 7 {
      let next_period = self.target_period(sequencer);
      if next_period < 0x0800 {
        sequencer.period = next_period;
        sequencer.frame_counter = next_period;
      }
    }

    if self.frame_counter > 0 && !self.is_reload {
      self.frame_counter = self.frame_counter.wrapping_sub(1);
    } else {
      self.frame_counter = self.current_period;
      self.is_reload = false;
    }
  }

  pub fn write_reg(&mut self, data: u8) {
    self.is_enabled = data & 0x80 > 0;
    self.current_period = (data & 0x70) >> 4;
    self.is_negate = data & 0x08 > 0;
    self.shift_amount = data & 0x07;
    self.is_reload = true;
  }

  pub fn target_period(&self, sequencer: &Sequencer) -> u16 {
    let period = sequencer.period;
    if self.is_negate {
      period - (period >> self.shift_amount) - self.negate_mode.value()
    } else {
      period + (period >> self.shift_amount)
    }
  }
}

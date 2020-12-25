use std::convert::TryFrom;

pub struct Sequencer {
  pub frame_counter: u16,
  pub period: u16,
  steps: usize,
  pub current_step: usize,
}

impl Sequencer {
  pub fn new(steps: usize) -> Sequencer {
    Sequencer {
      frame_counter: 0,
      period: 0,
      steps,
      current_step: 0
    }
  }

  pub fn step(&mut self, is_enabled: bool) -> bool {
    if self.frame_counter == 0 {
      self.frame_counter = self.period;
      if is_enabled {
        self.current_step = (self.current_step + 1) % self.steps;
      }
      true
    } else {
      self.frame_counter = self.frame_counter.wrapping_sub(1);
      false
    }
  }

  pub fn set_period_lo(&mut self, val: u8) {
    self.period = (self.period & 0xFF00) | u16::try_from(val).unwrap();
  }

  pub fn set_period_hi(&mut self, val: u8) {
    self.period = (self.period & 0x00FF) | ((u16::try_from(val).unwrap() & 0x07) << 8);
  }
}

use crate::apu::sequencer::Sequencer;
use crate::apu::length_counter::LengthCounter;

pub const SEQUENCE_LOOKUP_TABLE: [u8; 32] = [
  15, 14, 13, 12, 11, 10, 9,
  8, 7, 6, 5, 4, 3, 2, 1, 0,
  0, 1, 2, 3, 4, 5, 6, 7, 8,
  9, 10, 11, 12, 13, 14, 15,
];

pub struct Triangle {
  ctrl_flag: bool,
  sequencer: Sequencer,
  length_counter: LengthCounter,
  linear_counter: u8,
  is_linear_counter: bool,
  linear_counter_period: u8,

}

impl Triangle {
  pub fn new() -> Triangle {
    Triangle {
      ctrl_flag: false,
      sequencer: Sequencer::new(SEQUENCE_LOOKUP_TABLE.len()),
      length_counter: LengthCounter::new(),
      linear_counter: 0,
      is_linear_counter: false,
      linear_counter_period: 0
    }
  }

  pub fn triangle_write_reg_u8(&mut self, address: u16, data: u8) {
    match address {
      0x4008 => {
        self.ctrl_flag = data & 0x80 > 0;
        self.length_counter.set_halted(data & 0x80 > 0);
        self.linear_counter_period = data & 0x7F;
      },
      0x4009 => (),
      0x400A => self.sequencer.set_period_lo(data),
      0x400B => {
        self.length_counter.write_register(data);
        self.sequencer.set_period_hi(data & 0x07);
        self.is_linear_counter = true;
      }
      _ => panic!("Invalid triangle_write_reg address 0x{:04X}", address),
    }
  }

  pub fn sample(&self) -> u8 {
    if self.is_active() && self.sequencer.period > 2 {
      SEQUENCE_LOOKUP_TABLE[self.sequencer.current_step]
    } else {
      0
    }
  }

  pub fn step_sequencer(&mut self) {
    let is_active = self.is_active();
    self.sequencer.step(is_active);
  }

  pub fn step_quarter_frame(&mut self) {
    if self.is_linear_counter {
      self.linear_counter = self.linear_counter_period;
    } else if self.linear_counter > 0 {
      self.linear_counter -= 1;
    }

    if !self.ctrl_flag {
      self.is_linear_counter = false;
    }
  }

  pub fn step_half_frame(&mut self) {
    self.length_counter.step();
  }

  pub fn is_playing(&self) -> bool {
    self.length_counter.playing()
  }

  fn is_active(&self) -> bool {
    self.length_counter.active() && self.linear_counter > 0
  }

  pub fn set_enabled(&mut self, value: bool) {
    self.length_counter.set_enabled(value);
  }

  pub fn update_length_counter(&mut self) {
    self.length_counter.update_pending();
  }
}

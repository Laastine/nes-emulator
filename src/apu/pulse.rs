
use crate::apu::envelope::Envelope;
use crate::apu::length_counter::LengthCounter;
use crate::apu::sequencer::Sequencer;
use crate::apu::sweep::{Mode, Sweep};

const SEQUENCE_LOOKUP_TABLE: [[u8; 8]; 4] = [
  [0, 1, 0, 0, 0, 0, 0, 0],
  [0, 1, 1, 0, 0, 0, 0, 0],
  [0, 1, 1, 1, 1, 0, 0, 0],
  [1, 0, 0, 1, 1, 1, 1, 1],
];

pub struct Pulse {
  envelope: Envelope,
  sweep: Sweep,
  sequencer: Sequencer,
  length_counter: LengthCounter,
  cycle: usize,
}

impl Pulse {
  pub fn new(channel: Mode) -> Pulse {
    Pulse {
      envelope: Envelope::new(),
      sweep: Sweep::new(channel),
      sequencer: Sequencer::new(SEQUENCE_LOOKUP_TABLE[0].len()),
      length_counter: LengthCounter::new(),
      cycle: 0,
    }
  }

  pub fn pulse_write_reg_u8(&mut self, address: u16, data: u8) {
    match address % 4 {
      0x00 => {
        self.cycle = usize::from(data) >> 6;
        self.envelope.write_reg(data);
        self.length_counter.set_halted((data & 0x20) > 0)
      }
      0x01 => {
        self.sweep.write_reg(data);
      }
      0x02 => {
        self.sequencer.set_period_lo(data);
      }
      0x03 => {
        self.length_counter.write_register(data);
        self.sequencer.set_period_hi(data & 0x07);
        self.envelope.start();
        self.sequencer.current_step = 0;
      }
      _ => panic!("Invalid pulse_write_reg_u8 address 0x{:04X}", address),
    }
  }

  pub fn sample(&self) -> u8 {
    if self.length_counter.active() && self.sequencer.period > 7 && self.sweep.target_period(&self.sequencer) < 0x0800 {
      SEQUENCE_LOOKUP_TABLE[self.cycle][self.sequencer.current_step] * self.envelope.get_volume_level()
    } else {
      0
    }
  }

  pub fn step_quarter_frame(&mut self) {
    self.envelope.step();
  }

  pub fn step_half_frame(&mut self) {
    self.length_counter.step();
    self.sweep.step(&mut self.sequencer);
  }

  pub fn step_sequencer(&mut self) {
    self.sequencer.step(true);
  }

  pub fn is_playing(&mut self) -> bool {
    self.length_counter.playing()
  }

  pub fn set_enabled(&mut self, value: bool) {
    self.length_counter.set_enabled(value);
  }

  pub fn update_length_counter(&mut self) {
    self.length_counter.update_pending();
  }
}

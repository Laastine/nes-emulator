use std::convert::TryFrom;

use crate::apu::{pulse::Pulse, sweep::Mode};
use crate::apu::signal_filter::SignalFilter;
use crate::apu::frame_counter::{FrameCounter, FrameResult};

pub mod audio_stream;
mod envelope;
mod signal_filter;
mod frame_counter;
mod length_counter;
mod pulse;
mod sequencer;
mod sweep;

pub struct Apu {
  pub buf: Vec<f32>,
  filters: [SignalFilter; 3],
  pub pulse_0: Pulse,
  pub pulse_1: Pulse,
  frame_counter: FrameCounter
}

const AUDIO_BUFFER_LIMIT: usize = 1470;

impl Apu {
  pub fn new() -> Apu {

    Apu {
      buf: Vec::new(),
      frame_counter: FrameCounter::new(),
      pulse_0: Pulse::new(Mode::OnesComplement),
      pulse_1: Pulse::new(Mode::TwosComplement),
      filters: [
        SignalFilter::hi_pass(44100.0, 90.0),
        SignalFilter::hi_pass(44100.0, 440.0),
        SignalFilter::lo_pass(44100.0, 14_000.0),
      ]
    }
  }

  pub fn reset(&mut self) {
    self.apu_write_reg(0x4017, 0, 0);
    for idx in 0..0x0B {
      self.step(idx);
    }
  }

  pub fn step(&mut self, cycle: u32) {
    if cycle % 2 == 1 {
      self.pulse_0.step_sequencer();
      self.pulse_1.step_sequencer();
    }
    let frame_res = self.frame_counter.step();
    self.handle_frame_result(frame_res);

    self.pulse_0.update_length_counter();
    self.pulse_1.update_length_counter();

    if cycle % 40 == 0 && self.buf.len() < AUDIO_BUFFER_LIMIT {
      let sample = self.sample();
      self.buf.push(sample);
      self.buf.push(sample);
    }
  }

  pub fn apu_read_reg(&mut self) -> u8 {
    let mut res = 0;
    if self.frame_counter.private_irq_flag {
      res |= 0x40;
    }
    if self.pulse_1.playing() {
      res |= 0x02;
    }
    if self.pulse_0.playing() {
      res |= 0x01;
    }
    self.frame_counter.private_irq_flag = false;
    self.frame_counter.public_irq_flag = false;
    res
  }

  pub fn apu_write_reg(&mut self, address: u16, data: u8, cycle: u32) {
    match address {
      0x4000..=0x4003 => self.pulse_0.pulse_write_reg_u8(address , data),
      0x4004..=0x4007 => self.pulse_1.pulse_write_reg_u8(address , data),
      0x4008..=0x4013 => (),
      0x4015 => {
        self.pulse_0.set_enabled(data & 0x01 > 0);
        self.pulse_1.set_enabled(data & 0x02 > 0);
      },
      0x4017 => {
        let res = self.frame_counter.write_register(data, cycle);
        self.handle_frame_result(res);
      }
      _ => panic!("Invalid write_reg address 0x{:04X}", address),
    }
  }

  fn handle_frame_result(&mut self, res: FrameResult) {
    match res {
      FrameResult::Quarter => {
        self.pulse_0.step_quarter_frame();
        self.pulse_1.step_quarter_frame();
      },
      FrameResult::Half => {
        self.pulse_0.step_quarter_frame();
        self.pulse_0.step_half_frame();
        self.pulse_1.step_quarter_frame();
        self.pulse_1.step_half_frame();
      },
      FrameResult::None => (),
    }
  }

  pub fn get_irq_flag(&self) -> bool {
    self.frame_counter.public_irq_flag
  }

  fn sample(&mut self) -> f32 {
    let p0 = f64::try_from(self.pulse_0.sample()).unwrap();
    let p1 = f64::try_from(self.pulse_1.sample()).unwrap();

    let mut output = (95.88 / ((8218.0 / (p0 + p1)) + 100.0)) * 65535.0;

    for i in 0..3 {
      output = self.filters[i].step(output );
    }

    output as f32
  }
}

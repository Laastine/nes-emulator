use std::f64::consts::PI;

pub struct SignalFilter {
  b_0: f64,
  b_1: f64,
  a_1: f64,
  prev_x: f64,
  prev_y: f64,
}

impl SignalFilter {
  pub fn hi_pass(sample_rate: f64, cutoff_freq: f64) -> Self {
    let c = sample_rate / PI / cutoff_freq;
    let a0i = 1.0 / (1.0 + c);

    SignalFilter {
      b_0: c * a0i,
      b_1: -c * a0i,
      a_1: (1.0 - c) * a0i,
      prev_x: 0.0,
      prev_y: 0.0,
    }
  }

  pub fn lo_pass(sample_rate: f64, cutoff_freq: f64) -> Self {
    let c = sample_rate / PI / cutoff_freq;
    let a0i = 1.0 / (1.0 + c);

    SignalFilter {
      b_0: a0i,
      b_1: a0i,
      a_1: (1.0 - c) * a0i,
      prev_x: 0.0,
      prev_y: 0.0,
    }
  }

  pub fn step(&mut self, x: f64) -> f64 {
    let y = self.b_0 * x + self.b_1 * self.prev_x - self.a_1 * self.prev_y;
    self.prev_y = y;
    self.prev_x = x;
    y
  }
}

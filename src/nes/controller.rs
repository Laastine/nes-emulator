#[derive(Copy, Clone)]
pub struct Controller {
  input_states: [bool; 8],
  idx: usize,
  strobe: u8,
}

impl Controller {
  pub fn new() -> Controller {
    Controller {
      input_states: [false; 8],
      idx: 0,
      strobe: 0
    }
  }

  #[inline]
  pub fn update_buttons(&mut self, states: [bool; 8]) {
    for (idx, c) in self.input_states.iter_mut().enumerate() {
      *c = states[idx]
    }
  }

  pub fn read(&mut self) -> u8 {
    let mut value = 0;

    if self.idx < 8 && self.input_states[self.idx] {
      value = 1;
    }

    self.idx += 1;
    if self.strobe & 1 == 1 {
      self.idx = 0;
    }

    value
  }

  pub fn write(&mut self, val: u8) {
    self.strobe = val;

    if self.strobe & 1 == 1 {
      self.idx = 0;
    }
  }
}

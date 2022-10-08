#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Mode {
  Zero,
  One,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FrameResult {
  None,
  Quarter,
  Half,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct FrameCounter {
  pub counter: i32,
  pub cycles: u32,
  pub irq_enabled: bool,
  pub public_irq_flag: bool,
  pub private_irq_flag: bool,
  mode: Mode,
}

impl FrameCounter {
  pub fn new() -> Self {
    FrameCounter {
      counter: 0,
      cycles: 0,
      irq_enabled: true,
      public_irq_flag: false,
      private_irq_flag: false,
      mode: Mode::Zero,
    }
  }

  pub fn write_register(&mut self, value: u8, cycles: u32) -> FrameResult {
    self.irq_enabled = value & 0x40 == 0;
    if !self.irq_enabled {
      self.public_irq_flag = false;
      self.private_irq_flag = false;
    }

    self.mode = if value & 0x80 == 0 {
      Mode::Zero
    } else {
      Mode::One
    };

    self.counter = if cycles % 2 == 0 { 0 } else { -1 };

    match self.mode {
      Mode::Zero => FrameResult::None,
      Mode::One => FrameResult::Half,
    }
  }

  pub fn step(&mut self) -> FrameResult {
    let result = match self.mode {
      Mode::Zero => self.tick_mode_zero(),
      Mode::One => self.tick_mode_one(),
    };
    self.counter = self.counter.wrapping_add(1);
    result
  }

  fn tick_mode_zero(&mut self) -> FrameResult {
    match self.counter {
      0x1D23 => FrameResult::Quarter,
      0x3A43 => FrameResult::Half,
      0x5765 => FrameResult::Quarter,
      0x7486 => {
        self.trigger_irq();
        FrameResult::None
      }
      0x7487 => {
        self.trigger_irq();
        self.publish_irq();
        FrameResult::Half
      }
      0x7488 => {
        self.trigger_irq();
        self.publish_irq();
        self.counter = 2;
        FrameResult::None
      }
      _ => FrameResult::None,
    }
  }

  fn tick_mode_one(&mut self) -> FrameResult {
    match self.counter {
      0x1D23 => FrameResult::Quarter,
      0x3A43 => FrameResult::Half,
      0x5765 => FrameResult::Quarter,
      0x91A3 => {
        self.counter = 1;
        FrameResult::Half
      }
      _ => FrameResult::None,
    }
  }

  pub fn trigger_irq(&mut self) {
    if self.irq_enabled {
      self.private_irq_flag = true;
    }
  }
  pub fn publish_irq(&mut self) {
    self.public_irq_flag = self.private_irq_flag;
  }
}

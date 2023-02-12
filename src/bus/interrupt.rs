#[derive(Copy, Clone)]
pub struct Interrupt {
  schedule: Option<u8>,
}

impl Interrupt {
  pub fn new() -> Self {
    Interrupt { schedule: None }
  }

  pub fn tick(&mut self) {
    match self.schedule.as_mut() {
      Some(v) => if *v > 0 {
        *v -= 1
      },
      None => (),
    };
  }

  pub fn schedule(&mut self, n: u8) {
    self.schedule = Some(n);
  }

  pub fn clear(&mut self) {
    self.schedule = None;
  }

  pub fn ready(&self) -> bool {
    match self.schedule {
      Some(v) => v == 0,
      None => false,
    }
  }
}

extern crate termion;

use std::env;

use crate::bus::Bus;
use crate::nes::Nes;

mod bus;
mod cpu;
mod nes;
mod rom;

fn main() {
  let args: Vec<String> = env::args().collect();
  println!("Args {:?}", args);

  let mut bus = Bus::new();
  let mut nes = Nes::new(&mut bus);

  nes.create();
  nes.user_update();
}


use crate::nes::Nes;
use crate::rom::Rom;

mod cpu;
mod nes;
mod rom;

fn main() {
  let rom = Rom::read_from_file("foo");
}

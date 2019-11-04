use crate::nes::Nes;
use crate::rom::Rom;

use std::env;

mod bus;
mod cpu;
mod nes;
mod rom;

fn main() {
  let args: Vec<String> = env::args().collect();
  let rom_file = &args[1];
  let rom = example_rom();
  let mut nes = Nes::new(rom);
  nes.exec()
}

fn example_rom() -> Rom {
  let interrupt_vectors = vec![0x00, 0x00, 0x00, 0x80, 0x00, 0x00];
  let data = vec![
    0xA9, 0x05, 0x69, 0x06, 0xAA, 0x86, 0x01, 0xA5, 0x01, 0x4C, 0x09, 0x80,
  ];

  let rom = data
    .into_iter()
    .chain(std::iter::repeat(0x0))
    .take(0x4000 - interrupt_vectors.len())
    .chain(interrupt_vectors)
    .collect::<Vec<u8>>();

  Rom { rom }
}

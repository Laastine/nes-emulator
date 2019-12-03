extern crate getopts;
extern crate termion;

use std::env;

use crate::bus::Bus;
use crate::nes::Nes;
use crate::rom::Rom;
use getopts::Options;

mod bus;
mod cartridge;
mod cpu;
mod nes;
mod rom;

fn print_usage() {
  println!("USAGE:\nnes-emulator [FLAGS]\n\nFLAGS:\n-h, --help\t\t\tPrints help information\n-v, --version\t\t\tPrints version information\n-r, --rom\t\t\tRom filename to load");
}

fn print_version() {
  println!("0.1.0");
}

fn main() {
  let args: Vec<String> = env::args().collect();
  println!("Args {:?}", args);

  let mut opts = Options::new();
  opts.optflag("r", "rom", "ROM file name");
  opts.optflag("h", "help", "print help");
  opts.optflag("v", "version", "print version number");
  let matches = match opts.parse(&args[1..]) {
    Ok(m) => m,
    Err(e) => panic!(e.to_string()),
  };

  if matches.opt_present("h") {
    print_usage();
    return;
  }

  if matches.opt_present("v") {
    print_version();
    return;
  }

  let mut bus = Bus::new();
  let mut nes = Nes::new(&mut bus);

  nes.create_program();
  nes.render_loop();
}

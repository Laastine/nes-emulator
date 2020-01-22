#[macro_use]
extern crate bitfield;
extern crate getopts;
extern crate image;
#[cfg(target_family = "unix")]
#[cfg(feature = "terminal_debug")]
extern crate termion;

use std::env;

use getopts::Options;

use crate::nes::Nes;

mod bus;
mod cartridge;
mod cpu;
mod gfx;
mod mapper;
mod nes;
mod ppu;
#[cfg(target_family = "unix")]
#[cfg(feature = "terminal_debug")]
mod terminal_debug;

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

  let rom_file = if !matches.free.is_empty() {
    matches.free[0].clone()
  } else {
    panic!("No ROM file parameter given")
  };

  let mut nes = Nes::new(&rom_file);

  nes.init();
  nes.render_loop();
}

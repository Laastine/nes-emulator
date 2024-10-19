#[macro_use]
extern crate bitfield;
extern crate glium;
extern crate getopts;
extern crate image;


use std::env;

use getopts::Options;

use crate::nes::Nes;

mod apu;
mod bus;
mod cartridge;
mod cpu;
mod mapper;
mod nes;
mod ppu;
mod gfx;

fn main() {
  let args: Vec<String> = env::args().collect();

  let mut opts = Options::new();
  opts.optflag("r", "rom", "ROM file name");
  opts.optflag("h", "help", "print help");
  opts.optflag("d", "debug", "show memory debug");
  opts.optflag("v", "version", "print version number");
  let matches = match opts.parse(&args[1..]) {
    Ok(m) => m,
    Err(e) => panic!("{}", e.to_string()),
  };

  if matches.opt_present("h") {
    println!("USAGE:\nnes-emulator [FLAGS]\n\nFLAGS:\n-h, --help\t\t\tPrints help information\n-v, --version\t\t\tPrints version information\n-r, --rom\t\t\tRom filename to load\n-d, --debug\t\t\tShow memory debug on terminal");
    return;
  }

  if matches.opt_present("v") {
    println!("0.1.0");
    return;
  }

  let rom_file = if !matches.free.is_empty() {
    matches.free[0].to_string()
  } else {
    panic!("No ROM file parameter given")
  };

  let use_debug_mode = matches.opt_present("d");
  let mut nes = Nes::new(&rom_file, use_debug_mode);

  nes.reset();
  nes.render_loop();
}

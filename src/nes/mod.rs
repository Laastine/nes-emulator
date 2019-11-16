use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{stdin, stdout, Stdout, Write};
use std::str::FromStr;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use crate::bus::Bus;
use crate::cpu::{Cpu, hex};
use crate::cpu::instruction_table::FLAGS6502;

pub struct Nes<'a> {
  cpu: Cpu<'a>,
  map_asm: HashMap<u16, String>,
}

impl<'a> Nes<'a> {
  pub fn new(bus: &'a mut Bus) -> Nes<'a> {
    let cpu = Cpu::new(bus);

    let map_asm: HashMap<u16, String> = HashMap::new();

    Nes {
      cpu,
      map_asm,
    }
  }

  pub fn draw_ram(&mut self, stdout: &mut RawTerminal<Stdout>, x: u16, mut y: u16, mut addr: u16) {
    for row in 0..16 {
      let mut offset = format!("$:{}", hex(addr as usize, 4));
      for col in 0..16 {
        offset = format!("{}{}", offset, hex(self.cpu.bus.read_u8(addr).try_into().unwrap(), 2));
        addr += 1;
      }
      write!(stdout, "{}{}", termion::cursor::Goto(x, y), offset).unwrap();
      y += 1;
    }
  }

  pub fn draw_cpu(&self, stdout: &mut RawTerminal<Stdout>, x: u16, y: u16) {
    write!(stdout, "{}Status", termion::cursor::Goto(x, y)).unwrap();
    write!(stdout, "{}N {}", termion::cursor::Goto(x + 64, y), if self.cpu.status_register & FLAGS6502::N.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    write!(stdout, "{}V {}", termion::cursor::Goto(x + 80, y), if self.cpu.status_register & FLAGS6502::V.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    write!(stdout, "{}- {}", termion::cursor::Goto(x + 96, y), if self.cpu.status_register & FLAGS6502::U.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    write!(stdout, "{}B {}", termion::cursor::Goto(x + 112, y), if self.cpu.status_register & FLAGS6502::B.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    write!(stdout, "{}D {}", termion::cursor::Goto(x + 128, y), if self.cpu.status_register & FLAGS6502::D.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    write!(stdout, "{}I {}", termion::cursor::Goto(x + 144, y), if self.cpu.status_register & FLAGS6502::I.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    write!(stdout, "{}Z {}", termion::cursor::Goto(x + 160, y), if self.cpu.status_register & FLAGS6502::Z.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    write!(stdout, "{}C {}", termion::cursor::Goto(x + 178, y), if self.cpu.status_register & FLAGS6502::C.value() > 0x00 { "ON" } else { "OFF" }).unwrap();

    write!(stdout, "{}PC: ${}", termion::cursor::Goto(x, y + 10), hex(self.cpu.pc.try_into().unwrap(), 4)).unwrap();
    write!(stdout, "{}A: ${} {}", termion::cursor::Goto(x, y + 20), hex(self.cpu.a.try_into().unwrap(), 2), self.cpu.a).unwrap();
    write!(stdout, "{}X: ${} {}", termion::cursor::Goto(x, y + 30), hex(self.cpu.x.try_into().unwrap(), 2), self.cpu.x).unwrap();
    write!(stdout, "{}Y: ${} {}", termion::cursor::Goto(x, y + 40), hex(self.cpu.y.try_into().unwrap(), 2), self.cpu.y).unwrap();

    write!(stdout, "{}Stack P: ${}", termion::cursor::Goto(x, y + 50), hex(usize::from(self.cpu.stack_pointer), 4)).unwrap();
  }

  pub fn draw_code(&self, stdout: &mut RawTerminal<Stdout>, x: u16, y: u16, lines: u16) {
    let val = self.map_asm.get(&self.cpu.pc).unwrap();
    write!(stdout, "{}{}", termion::cursor::Goto(x, y), val).unwrap();
  }

  pub fn create(&mut self) {
    let hex_str = "A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA";
    let mut offset = 0x8000;
    for s in hex_str.split_ascii_whitespace() {
      offset += 1;
      self.cpu.bus.memory[offset] = u8::from_str(s).unwrap_or(0);
    }

    self.cpu.bus.memory[0xFFFC] = 0x00;
    self.cpu.bus.memory[0xFFFD] = 0x80;

    self.map_asm = self.cpu.disassemble(0x0000, 0xFFFF);

    self.cpu.reset();
  }

  pub fn user_update(&mut self) {
    let stdin = stdin();
    let mut stdout = stdout()
      .into_raw_mode()
      .unwrap_or_else(|err| panic!("stdout raw mode error {:?}", err));

    for c in stdin.keys() {
      match c.unwrap() {
        Key::Char('q') | Key::Esc => break,
        Key::Right => {
          while !self.cpu.complete() {
            self.cpu.clock();
          }
        }
        Key::Char('r') => {
          self.cpu.reset();
        }
        Key::Char('i') => {
          self.cpu.irq();
        }
        Key::Char('n') => {
          self.cpu.nmi();
        }
        _ => (),
      }

      self.draw_ram(&mut stdout, 0x0000, 16, 16);
      self.draw_ram(&mut stdout, 0x8000, 16, 16);
      self.draw_cpu(&mut stdout, 448, 2);
      self.draw_code(&mut stdout, 448, 72, 26);
    }
  }
}

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

  pub fn draw_ram(&mut self, stdout: &mut RawTerminal<Stdout>, mut addr: u16, x: u16, y: u16, rows: u16, columns: u16) {
    let mut y_ram = y;
    let x_ram = x;
    for _ in 0..rows {
      let mut offset = format!("${}:", hex(addr as usize, 4));
      for _ in 0..columns {
        offset = format!("{} {}", offset, hex(self.cpu.bus.read_u8(addr).try_into().unwrap(), 2));
        addr += 1;
      }
      write!(stdout, "{}{}", termion::cursor::Goto(x_ram, y_ram), offset).unwrap();
      y_ram += 1;
    }
  }

  pub fn draw_cpu(&self, stdout: &mut RawTerminal<Stdout>, x: u16, y: u16) {
    writeln!(stdout, "{}Status", termion::cursor::Goto(x, y)).unwrap();
    writeln!(stdout, "{}N {}", termion::cursor::Goto(x + 1, y), if self.cpu.status_register & FLAGS6502::N.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    writeln!(stdout, "{}V {}", termion::cursor::Goto(x + 2, y), if self.cpu.status_register & FLAGS6502::V.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    writeln!(stdout, "{}- {}", termion::cursor::Goto(x + 3, y), if self.cpu.status_register & FLAGS6502::U.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    writeln!(stdout, "{}B {}", termion::cursor::Goto(x + 4, y), if self.cpu.status_register & FLAGS6502::B.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    writeln!(stdout, "{}D {}", termion::cursor::Goto(x + 5, y), if self.cpu.status_register & FLAGS6502::D.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    writeln!(stdout, "{}I {}", termion::cursor::Goto(x + 6, y), if self.cpu.status_register & FLAGS6502::I.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    writeln!(stdout, "{}Z {}", termion::cursor::Goto(x + 7, y), if self.cpu.status_register & FLAGS6502::Z.value() > 0x00 { "ON" } else { "OFF" }).unwrap();
    writeln!(stdout, "{}C {}", termion::cursor::Goto(x + 8, y), if self.cpu.status_register & FLAGS6502::C.value() > 0x00 { "ON" } else { "OFF" }).unwrap();

    writeln!(stdout, "{}PC: ${}", termion::cursor::Goto(x, y + 1), hex(self.cpu.pc.try_into().unwrap(), 4)).unwrap();
    writeln!(stdout, "{}A: ${} {}", termion::cursor::Goto(x, y + 2), hex(self.cpu.a.try_into().unwrap(), 2), self.cpu.a).unwrap();
    writeln!(stdout, "{}X: ${} {}", termion::cursor::Goto(x, y + 3), hex(self.cpu.x.try_into().unwrap(), 2), self.cpu.x).unwrap();
    writeln!(stdout, "{}Y: ${} {}", termion::cursor::Goto(x, y + 4), hex(self.cpu.y.try_into().unwrap(), 2), self.cpu.y).unwrap();

    writeln!(stdout, "{}Stack P: ${}", termion::cursor::Goto(x, y + 5), hex(usize::from(self.cpu.stack_pointer), 4)).unwrap();
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

    write!(stdout, "{}{}", termion::cursor::Goto(1, 1), termion::clear::AfterCursor).unwrap();

    for c in stdin.keys() {
      match c.unwrap() {
        Key::Char('q') | Key::Esc => {
          write!(stdout, "{}{}", termion::cursor::Goto(1, 1), termion::clear::AfterCursor).unwrap();
          break;
        }
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

      self.draw_ram(&mut stdout, 0x0000, 2, 2, 16, 16);
      self.draw_ram(&mut stdout, 0x8000, 2, 20, 16, 16);
      self.draw_cpu(&mut stdout, 64, 2);
      self.draw_code(&mut stdout, 64, 12, 26);
    }
  }
}

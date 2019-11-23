use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{stdin, stdout, Stdout, Write};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{color, cursor};

use crate::bus::Bus;
use crate::cpu::instruction_table::FLAGS6502;
use crate::cpu::{hex, Cpu};

pub struct Nes<'a> {
  cpu: Cpu<'a>,
  map_asm: HashMap<u16, String>,
}

impl<'a> Nes<'a> {
  pub fn new(bus: &'a mut Bus) -> Nes<'a> {
    let cpu = Cpu::new(bus);

    let map_asm: HashMap<u16, String> = HashMap::new();

    Nes { cpu, map_asm }
  }

  fn draw_ram(
    &mut self,
    stdout: &mut RawTerminal<Stdout>,
    mut addr: u16,
    x: u16,
    y: u16,
    rows: u16,
    columns: u16,
  ) {
    let mut y_ram = y;
    let x_ram = x;
    for _ in 0..rows {
      let mut offset = format!("${}:", hex(addr as usize, 4));
      for _ in 0..columns {
        offset = format!(
          "{} {}",
          offset,
          hex(self.cpu.bus.read_u8(addr).try_into().unwrap(), 2)
        );
        addr += 1;
      }
      write!(stdout, "{}{}", termion::cursor::Goto(x_ram, y_ram), offset).unwrap();
      y_ram += 1;
    }
  }

  fn draw_cpu(&self, stdout: &mut RawTerminal<Stdout>, x: u16, y: u16) {
    write!(stdout, "{}Status", termion::cursor::Goto(x, y)).unwrap();
    let green = color::Fg(termion::color::AnsiValue::rgb(0, 5, 0));
    let red = color::Fg(termion::color::AnsiValue::rgb(5, 0, 0));
    if self.cpu.status_register & FLAGS6502::N.value() > 0x00 {
      write!(stdout, "{}{}N", cursor::Goto(x + 7, y), green).unwrap();
    } else {
      write!(stdout, "{}{}N", cursor::Goto(x + 7, y), red).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::V.value() > 0x00 {
      write!(stdout, "{}{}V", cursor::Goto(x + 9, y), green).unwrap();
    } else {
      write!(stdout, "{}{}V", cursor::Goto(x + 9, y), red).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::U.value() > 0x00 {
      write!(stdout, "{}{}-", cursor::Goto(x + 11, y), green).unwrap();
    } else {
      write!(stdout, "{}{}-", cursor::Goto(x + 11, y), red).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::B.value() > 0x00 {
      write!(stdout, "{}{}B", cursor::Goto(x + 13, y), green).unwrap();
    } else {
      write!(stdout, "{}{}B", cursor::Goto(x + 13, y), red).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::D.value() > 0x00 {
      write!(stdout, "{}{}D", cursor::Goto(x + 15, y), green).unwrap();
    } else {
      write!(stdout, "{}{}D", cursor::Goto(x + 15, y), red).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::I.value() > 0x00 {
      write!(stdout, "{}{}I", cursor::Goto(x + 17, y), green).unwrap();
    } else {
      write!(stdout, "{}{}I", cursor::Goto(x + 17, y), red).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::Z.value() > 0x00 {
      write!(stdout, "{}{}Z", cursor::Goto(x + 19, y), green).unwrap();
    } else {
      write!(stdout, "{}{}Z", cursor::Goto(x + 19, y), red).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::C.value() > 0x00 {
      write!(stdout, "{}{}C", cursor::Goto(x + 21, y), green).unwrap();
    } else {
      write!(stdout, "{}{}C", cursor::Goto(x + 21, y), red).unwrap();
    }
    writeln!(stdout, "{}", termion::style::Reset).unwrap();

    write!(
      stdout,
      "{}PC: ${}",
      cursor::Goto(x, y + 1),
      hex(self.cpu.pc.try_into().unwrap(), 4)
    )
    .unwrap();
    write!(
      stdout,
      "{}A: ${} [{}]",
      cursor::Goto(x, y + 2),
      hex(self.cpu.a.try_into().unwrap(), 2),
      self.cpu.a
    )
    .unwrap();
    write!(
      stdout,
      "{}X: ${} [{}]",
      cursor::Goto(x, y + 3),
      hex(self.cpu.x.try_into().unwrap(), 2),
      self.cpu.x
    )
    .unwrap();
    write!(
      stdout,
      "{}Y: ${} [{}]",
      cursor::Goto(x, y + 4),
      hex(self.cpu.y.try_into().unwrap(), 2),
      self.cpu.y
    )
    .unwrap();

    write!(
      stdout,
      "{}Stack P: ${}",
      cursor::Goto(x, y + 5),
      hex(usize::from(self.cpu.stack_pointer), 4)
    )
    .unwrap();
  }

  pub fn draw_code(&self, stdout: &mut RawTerminal<Stdout>, x: u16, y: u16) {
    let val = self.map_asm.get(&self.cpu.pc).unwrap();
    let blue = color::Fg(termion::color::AnsiValue::rgb(0, 0, 5));
    write!(
      stdout,
      "{}{}",
      cursor::Goto(x, y),
      termion::clear::AfterCursor
    )
    .unwrap();
    write!(
      stdout,
      "{}{}{}{}",
      cursor::Goto(x, y),
      blue,
      val,
      termion::style::Reset
    )
    .unwrap();
  }

  pub fn create(&mut self) {
    let hex_str =
      "A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA";
    let mut offset = 0x8000;
    for s in hex_str.split_ascii_whitespace() {
      self
        .cpu
        .bus
        .write_u8(offset, u8::from_str_radix(s, 16).unwrap());
      offset += 1;
    }

    self.cpu.bus.write_u8(0xFFFC, 0x00);
    self.cpu.bus.write_u8(0xFFFD, 0x80);

    self.map_asm = self.cpu.disassemble(0x0000, 0xFFFF);

    self.cpu.reset();
  }

  fn draw(&mut self, stdout: &mut RawTerminal<Stdout>) {
    self.draw_ram(stdout, 0x0000, 2, 2, 16, 16);
    self.draw_ram(stdout, 0x8000, 2, 20, 16, 16);
    self.draw_cpu(stdout, 64, 2);
    self.draw_code(stdout, 64, 9);
  }

  pub fn user_update(&mut self) {
    let stdin = stdin();
    let mut stdout = stdout()
      .into_raw_mode()
      .unwrap_or_else(|err| panic!("stdout raw mode error {:?}", err));

    write!(
      stdout,
      "{}{}",
      termion::cursor::Goto(1, 1),
      termion::clear::AfterCursor
    )
    .unwrap();

    self.draw(&mut stdout);

    for c in stdin.keys() {
      match c.unwrap() {
        Key::Char('q') | Key::Esc => {
          write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(1, 1),
            termion::clear::AfterCursor
          )
          .unwrap();
          break;
        }
        Key::Char('x') => loop {
          self.cpu.clock();
          if !self.cpu.complete() {
            break;
          }
        },
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
      self.draw(&mut stdout);
    }
  }
}

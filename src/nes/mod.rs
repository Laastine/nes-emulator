use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{stdin, stdout, Stdout, Write};

use termion::{clear, color, cursor, style};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use crate::bus::Bus;
use crate::cpu::{Cpu, hex};
use crate::cpu::instruction_table::FLAGS6502;

const RED: color::Fg<color::AnsiValue> = color::Fg(color::AnsiValue(196));
const GREEN: color::Fg<color::AnsiValue> = color::Fg(color::AnsiValue(46));
const BLUE: color::Fg<color::AnsiValue> = color::Fg(color::AnsiValue(21));

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
      write!(stdout, "{}{}", cursor::Goto(x_ram, y_ram), offset).unwrap();
      y_ram += 1;
    }
  }

  fn draw_cpu(&self, stdout: &mut RawTerminal<Stdout>, x: u16, y: u16) {
    write!(stdout, "{}Status", cursor::Goto(x, y)).unwrap();
    if self.cpu.status_register & FLAGS6502::N.value() > 0x00 {
      write!(stdout, "{}{}N", cursor::Goto(x + 7, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}N", cursor::Goto(x + 7, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::V.value() > 0x00 {
      write!(stdout, "{}{}V", cursor::Goto(x + 9, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}V", cursor::Goto(x + 9, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::U.value() > 0x00 {
      write!(stdout, "{}{}-", cursor::Goto(x + 11, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}-", cursor::Goto(x + 11, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::B.value() > 0x00 {
      write!(stdout, "{}{}B", cursor::Goto(x + 13, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}B", cursor::Goto(x + 13, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::D.value() > 0x00 {
      write!(stdout, "{}{}D", cursor::Goto(x + 15, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}D", cursor::Goto(x + 15, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::I.value() > 0x00 {
      write!(stdout, "{}{}I", cursor::Goto(x + 17, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}I", cursor::Goto(x + 17, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::Z.value() > 0x00 {
      write!(stdout, "{}{}Z", cursor::Goto(x + 19, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}Z", cursor::Goto(x + 19, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::C.value() > 0x00 {
      write!(stdout, "{}{}C", cursor::Goto(x + 21, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}C", cursor::Goto(x + 21, y), RED).unwrap();
    }
    writeln!(stdout, "{}", style::Reset).unwrap();

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
      hex(self.cpu.acc.try_into().unwrap(), 2),
      self.cpu.acc
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
    write!(
      stdout,
      "{}{}",
      cursor::Goto(x, y),
      clear::AfterCursor
    )
    .unwrap();
    write!(
      stdout,
      "{}{}{}{}",
      cursor::Goto(x, y),
      BLUE,
      val,
      style::Reset
    )
    .unwrap();
  }

  pub fn create_program(&mut self) {
    self.map_asm = self.cpu.disassemble(0x0000, 0xFFFF);
    self.cpu.reset();
  }

  fn draw_help(&mut self, stdout: &mut RawTerminal<Stdout>, x: u16, y: u16) {
    write!(stdout, "{}Exec next instruction: X\tIRQ: I\t\tNMI: N\t\tRESET: R", cursor::Goto(x,y)).unwrap();
  }

  fn draw(&mut self, stdout: &mut RawTerminal<Stdout>) {
    self.draw_ram(stdout, 0x0000, 2, 2, 16, 16);
    self.draw_ram(stdout, 0x8000, 2, 20, 16, 16);
    self.draw_cpu(stdout, 64, 2);
    self.draw_code(stdout, 64, 9);
    self.draw_help(stdout, 2, 37)
  }

  pub fn render_loop(&mut self) {
    let stdin = stdin();
    let mut stdout = stdout()
      .into_raw_mode()
      .unwrap_or_else(|err| panic!("stdout raw mode error {:?}", err));

    write!(
      stdout,
      "{}{}",
      cursor::Goto(1, 1),
      clear::AfterCursor
    )
    .unwrap();

    self.draw(&mut stdout);

    for c in stdin.keys() {
      match c.unwrap() {
        Key::Char('q') | Key::Esc => {
          write!(
            stdout,
            "{}{}",
            cursor::Goto(1, 1),
            clear::AfterCursor
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

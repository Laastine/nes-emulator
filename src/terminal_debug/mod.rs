use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Stdout;
use std::io::Write;

use termion::{clear, color, cursor, style};
use termion::raw::RawTerminal;

#[cfg(feature = "terminal_debug")]
use crate::bus::Bus;
#[cfg(feature = "terminal_debug")]
use crate::cpu::{Cpu, hex};
#[cfg(feature = "terminal_debug")]
use crate::cpu::instruction_table::FLAGS6502;

#[cfg(feature = "terminal_debug")]
struct TerminalDebug {
  map_asm: HashMap<u16, String>,
}

#[cfg(feature = "terminal_debug")]
impl TerminalDebug {
  pub fn new(cpu: &mut Cpu) -> TerminalDebug {
    let map_asm = cpu.disassemble(0x0000, 0xFFFF);

    TerminalDebug {
      map_asm
    }
  }

  #[cfg(feature = "terminal_debug")]
  #[cfg(target_family = "unix")]
  fn draw_ram(
    &mut self,
    bus: &mut Bus,
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
          hex(bus.read_u8(addr, true).try_into().unwrap(), 2)
        );
        addr += 1;
      }
      write!(stdout, "{}{}", cursor::Goto(x_ram, y_ram), offset).unwrap();
      y_ram += 1;
    }
  }

  #[cfg(feature = "terminal_debug")]
  #[cfg(target_family = "unix")]
  fn draw_cpu(&self, cpu: &Cpu, stdout: &mut RawTerminal<Stdout>, x: u16, y: u16) {
    const RED: color::Fg<color::AnsiValue> = color::Fg(color::AnsiValue(196));
    const GREEN: color::Fg<color::AnsiValue> = color::Fg(color::AnsiValue(46));

    write!(stdout, "{}Status", cursor::Goto(x, y)).unwrap();
    if cpu.status_register & FLAGS6502::N.value() > 0x00 {
      write!(stdout, "{}{}N", cursor::Goto(x + 7, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}N", cursor::Goto(x + 7, y), RED).unwrap();
    }
    if cpu.status_register & FLAGS6502::V.value() > 0x00 {
      write!(stdout, "{}{}V", cursor::Goto(x + 9, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}V", cursor::Goto(x + 9, y), RED).unwrap();
    }
    if cpu.status_register & FLAGS6502::U.value() > 0x00 {
      write!(stdout, "{}{}-", cursor::Goto(x + 11, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}-", cursor::Goto(x + 11, y), RED).unwrap();
    }
    if cpu.status_register & FLAGS6502::B.value() > 0x00 {
      write!(stdout, "{}{}B", cursor::Goto(x + 13, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}B", cursor::Goto(x + 13, y), RED).unwrap();
    }
    if cpu.status_register & FLAGS6502::D.value() > 0x00 {
      write!(stdout, "{}{}D", cursor::Goto(x + 15, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}D", cursor::Goto(x + 15, y), RED).unwrap();
    }
    if cpu.status_register & FLAGS6502::I.value() > 0x00 {
      write!(stdout, "{}{}I", cursor::Goto(x + 17, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}I", cursor::Goto(x + 17, y), RED).unwrap();
    }
    if cpu.status_register & FLAGS6502::Z.value() > 0x00 {
      write!(stdout, "{}{}Z", cursor::Goto(x + 19, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}Z", cursor::Goto(x + 19, y), RED).unwrap();
    }
    if cpu.status_register & FLAGS6502::C.value() > 0x00 {
      write!(stdout, "{}{}C", cursor::Goto(x + 21, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}C", cursor::Goto(x + 21, y), RED).unwrap();
    }
    writeln!(stdout, "{}", style::Reset).unwrap();

    write!(
      stdout,
      "{}PC: ${}",
      cursor::Goto(x, y + 1),
      hex(cpu.pc.try_into().unwrap(), 4)
    )
      .unwrap();
    write!(
      stdout,
      "{}A: ${} [{}]",
      cursor::Goto(x, y + 2),
      hex(cpu.acc.try_into().unwrap(), 2),
      cpu.acc
    )
      .unwrap();
    write!(
      stdout,
      "{}X: ${} [{}]",
      cursor::Goto(x, y + 3),
      hex(cpu.x.try_into().unwrap(), 2),
      cpu.x
    )
      .unwrap();
    write!(
      stdout,
      "{}Y: ${} [{}]",
      cursor::Goto(x, y + 4),
      hex(cpu.y.try_into().unwrap(), 2),
      cpu.y
    )
      .unwrap();

    write!(
      stdout,
      "{}Stack P: ${}",
      cursor::Goto(x, y + 5),
      hex(usize::from(cpu.stack_pointer), 4)
    )
      .unwrap();
  }

  #[cfg(feature = "terminal_debug")]
  #[cfg(target_family = "unix")]
  fn draw_code(&self, cpu: &Cpu, stdout: &mut RawTerminal<Stdout>, x: u16, y: u16) {
    let val = self.map_asm.get(&cpu.pc);
    if let Some(value) = val {
      write!(stdout, "{}{}", cursor::Goto(x, y), clear::AfterCursor).unwrap();
      write!(
        stdout,
        "{}{}{}",
        cursor::Goto(x, y),
        value,
        style::Reset
      )
        .unwrap();
    }
  }

  #[cfg(feature = "terminal_debug")]
  #[cfg(target_family = "unix")]
  pub fn draw_terminal(&mut self, cpu: &mut Cpu, stdout: &mut RawTerminal<Stdout>) {
    self.draw_ram(&mut cpu.bus, stdout, 0x0000, 2, 2, 16, 16);
    self.draw_ram(&mut cpu.bus, stdout, 0x8000, 2, 20, 16, 16);
    self.draw_cpu(cpu, stdout, 64, 2);
    self.draw_code(cpu, stdout, 64, 9);
  }
}

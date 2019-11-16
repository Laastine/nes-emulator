use std::collections::HashMap;
use std::str::FromStr;

use std::io::{stdout, stdin};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

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

  pub fn draw_ram(&mut self, mut addr: u16) {
    println!("draw_ram");
    for row in 0..16 {
      let mut offset = format!("$:{}", hex(addr as usize, 4));
      for col in 0..16 {
        offset = format!("{} {}", offset, hex(usize::from(self.cpu.bus.read_u8(addr)), 2));
        addr += 1;
      }
    }
  }

  pub fn draw_cpu(&self) {
    println!("draw_cpu");
    println!("N {}", if self.cpu.status_register & FLAGS6502::N.value() > 0x00 { "ON" } else { "OFF" });
    println!("V {}", if self.cpu.status_register & FLAGS6502::V.value() > 0x00 { "ON" } else { "OFF" });
    println!("- {}", if self.cpu.status_register & FLAGS6502::U.value() > 0x00 { "ON" } else { "OFF" });
    println!("B {}", if self.cpu.status_register & FLAGS6502::B.value() > 0x00 { "ON" } else { "OFF" });
    println!("D {}", if self.cpu.status_register & FLAGS6502::D.value() > 0x00 { "ON" } else { "OFF" });
    println!("I {}", if self.cpu.status_register & FLAGS6502::I.value() > 0x00 { "ON" } else { "OFF" });
    println!("Z {}", if self.cpu.status_register & FLAGS6502::Z.value() > 0x00 { "ON" } else { "OFF" });
    println!("C {}", if self.cpu.status_register & FLAGS6502::C.value() > 0x00 { "ON" } else { "OFF" });

    println!("PC: ${}", hex(usize::from(self.cpu.pc), 4));
    println!("A: ${}  {}", hex(usize::from(self.cpu.a), 2), usize::from(self.cpu.a));
    println!("X: ${}  {}", hex(usize::from(self.cpu.x), 2), usize::from(self.cpu.x));
    println!("Y: ${}  {}", hex(usize::from(self.cpu.y), 2), usize::from(self.cpu.y));

    println!("Stack P: ${}", hex(usize::from(self.cpu.stack_pointer), 4));
  }

  pub fn draw_code(&self) {
    let val = self.map_asm.get(&self.cpu.pc);
    println!("draw_code {:?}", val);
  }

  pub fn create(&mut self) -> bool {
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
    true
  }

  pub fn user_update(&mut self) -> bool {
    let stdin = stdin();
    let mut stdout = stdout()
      .into_raw_mode()
      .unwrap_or_else(|err| panic!("stdout raw mode error {:?}", err));

    for c in stdin.keys() {
      match c.unwrap() {
        Key::Char('q') => break,
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

      self.draw_ram(0x0000);
      self.draw_ram(0x8000);
      self.draw_cpu();
      self.draw_code();
    }
    true
  }
}

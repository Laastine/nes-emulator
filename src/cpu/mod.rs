use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use crate::bus::Bus;
use crate::cpu::instruction_table::{ADDRMODE6502, FLAGS6502, LookUpTable, OPCODES6502};
use std::fs::OpenOptions;
use std::io::Write;

pub mod instruction_table;

#[cfg(debug_assertions)]
fn init_log_file() {
  let file = OpenOptions::new().write(true).append(false).open("log.txt").expect("File open error");
  file.set_len(0).unwrap();
}

fn bool_to_u8(input: bool) -> u8 {
  if input { 1 } else { 0 }
}

fn u8_to_bool(input: u8) -> bool {
  input > 0
}

pub struct Cpu {
  pub bus: Bus,
  pub pc: u16,
  pub acc: u8,
  pub x: u8,
  pub y: u8,
  pub status_register: u8,
  pub stack_pointer: u8,
  fetched: u8,
  temp: u16,
  addr_abs: u16,
  addr_rel: u16,
  pub opcode: u8,
  pub cycles: u8,
  pub clock_count: u32,
  lookup: LookUpTable,
}

impl Cpu {
  pub fn new(bus: Bus) -> Cpu {
    let lookup = LookUpTable::new();
    init_log_file();
    Cpu {
      bus,
      pc: 0,
      acc: 0,
      x: 0,
      y: 0,
      fetched: 0x0,
      temp: 0x0u16,
      addr_abs: 0x0u16,
      addr_rel: 0x0u16,
      opcode: 0x0u8,
      stack_pointer: 0x0u8,
      status_register: 0u8,
      cycles: 0u8,
      clock_count: 0,
      lookup,
    }
  }

  fn set_flag(&mut self, flag: &FLAGS6502, val: bool) {
    let f = flag.value();
    if val {
      self.status_register |= f;
    } else {
      self.status_register &= !f;
    }
  }

  pub fn get_flag(&self, flag: &FLAGS6502) -> bool {
    (self.status_register & flag.value()) > 0
  }

  fn bus_mut_read_u8(&mut self, address: u16) -> u16 {
    self.bus.read_u8(address, false).try_into().unwrap()
  }

  fn bus_read_u8(&mut self, address: u16) -> u16 {
    self.bus.read_u8(address, true).try_into().unwrap()
  }

  fn bus_write_u8(&mut self, address: u16, data: u8) {
    self.bus.write_u8(address, data);
  }

  fn get_stack_address(&self) -> u16 {
    0x0100 + u16::try_from(self.stack_pointer).unwrap()
  }

  fn pc_increment(&mut self) {
    self.pc = self.pc.wrapping_add(1);
  }

  fn cycles_increment(&mut self) {
    self.cycles = self.cycles.wrapping_add(1);
  }

  fn stack_pointer_increment(&mut self) {
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
  }

  fn stack_pointer_decrement(&mut self) {
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
  }

  pub fn clock(&mut self) {
    if self.cycles == 0 {
      self.opcode = u8::try_from(self.bus_mut_read_u8(self.pc)).unwrap();

      let log_pc = self.pc;

      self.set_flag(&FLAGS6502::U, true);
      self.pc_increment();

      let opcode_idx = usize::try_from(self.opcode).unwrap();
      self.cycles = self.lookup.get_cycles(opcode_idx);

      let addr_mode = *self.lookup.get_addr_mode(opcode_idx);
      let operate = *self.lookup.get_operate(opcode_idx);

      self.cycles += self.addr_mode_value(addr_mode) & self.op_code_value(operate);

      self.set_flag(&FLAGS6502::U, true);

      self.log(usize::try_from(log_pc).unwrap());
    }

    self.clock_count += 1;

    if self.clock_count > 200_000 {
      std::process::exit(0);
    }

    self.cycles -= 1;
  }

  #[cfg(debug_assertions)]
  fn log(&self, log_pc: usize) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("log.txt")
        .expect("File append error");

    file
        .write_all(
          format!(
            "opcode:{} -> clock:{} sreg:{} {},{} PC:{} XXX A:{} X:{} Y:{} {}{}{}{}{}{}{}{} STKP:{}\n",
            self.opcode,
            self.clock_count,
            self.status_register,
            self.addr_abs,
            self.addr_rel,
            hex(log_pc, 4),
            hex(u8::try_into(self.acc).unwrap(), 2),
            hex(u8::try_into(self.x).unwrap(), 2),
            hex(u8::try_into(self.y).unwrap(), 2),
            if self.get_flag(&FLAGS6502::N) { "N" } else { "." },
            if self.get_flag(&FLAGS6502::V) { "V" } else { "." },
            if self.get_flag(&FLAGS6502::U) { "U" } else { "." },
            if self.get_flag(&FLAGS6502::B) { "B" } else { "." },
            if self.get_flag(&FLAGS6502::D) { "D" } else { "." },
            if self.get_flag(&FLAGS6502::I) { "I" } else { "." },
            if self.get_flag(&FLAGS6502::Z) { "Z" } else { "." },
            if self.get_flag(&FLAGS6502::C) { "C" } else { "." },
            hex(usize::try_from(self.stack_pointer).unwrap(), 2),
          )
              .as_bytes(),
        )
        .expect("File write error");
  }

  pub fn fetch(&mut self) {
    if self.addr_mode() != ADDRMODE6502::IMP {
      self.fetched = u8::try_from(self.bus_mut_read_u8(self.addr_abs) & 0x00FF).unwrap();
    }
  }

  pub fn reset(&mut self) {
    self.addr_abs = 0xFFFC;

    let lo_byte = self.bus_mut_read_u8(self.addr_abs);
    let hi_byte = self.bus_mut_read_u8(self.addr_abs + 1);

    self.pc = (hi_byte << 8) | lo_byte;
    self.acc = 0;
    self.x = 0;
    self.y = 0;
    self.stack_pointer = 0xFD;
    self.status_register = 0x00 | FLAGS6502::U.value();

    self.addr_abs = 0x0000;
    self.addr_rel = 0x0000;
    self.fetched = 0x00;

    self.cycles = 8;
  }

  /// Interrupt
  pub fn irq(&mut self) {
    if !self.get_flag(&FLAGS6502::I) {
      self.bus_write_u8(self.get_stack_address(),
                        u8::try_from((self.pc >> 8) & 0x00FF).unwrap(),
      );
      self.stack_pointer_decrement();

      self.bus_write_u8(self.get_stack_address(),
                        u8::try_from(self.pc & 0x00FF).unwrap());
      self.stack_pointer_decrement();

      self.set_flag(&FLAGS6502::B, false);
      self.set_flag(&FLAGS6502::U, true);
      self.set_flag(&FLAGS6502::I, true);

      self.bus_write_u8(self.get_stack_address(), self.status_register);
      self.stack_pointer_decrement();

      self.addr_abs = 0xFFFE;
      let lo_byte = self.bus_mut_read_u8(self.addr_abs);
      let hi_byte = self.bus_mut_read_u8(self.addr_abs + 1);
      self.pc = u16::try_from((hi_byte << 8) | lo_byte).unwrap();

      self.cycles = 7;
    }
  }

  /// Non-maskable interrupt
  pub fn nmi(&mut self) {
    self.bus_write_u8(
      self.get_stack_address(),
      u8::try_from((self.pc >> 8) & 0x00FF).unwrap(),
    );
    self.stack_pointer_decrement();

    self.bus_write_u8(
      self.get_stack_address(),
      u8::try_from(self.pc & 0x00FF).unwrap(),
    );
    self.stack_pointer_decrement();

    self.set_flag(&FLAGS6502::B, false);
    self.set_flag(&FLAGS6502::U, true);
    self.set_flag(&FLAGS6502::I, true);
    self.bus_write_u8(
      self.get_stack_address(),
      self.status_register,
    );
    self.stack_pointer_decrement();

    self.addr_abs = 0xFFFA;
    let lo_byte = self.bus_mut_read_u8(self.addr_abs);
    let hi_byte = self.bus_mut_read_u8(self.addr_abs + 1);
    self.pc = (hi_byte << 8) | lo_byte;

    self.cycles = 8;
  }

  /// ADDRESS MODES
  pub fn addr_mode_value(&mut self, addr_mode: ADDRMODE6502) -> u8 {
    match addr_mode {
      ADDRMODE6502::IMP => self.imp(),
      ADDRMODE6502::IMM => self.imm(),
      ADDRMODE6502::ZP0 => self.zp0(),
      ADDRMODE6502::ZPX => self.zpx(),
      ADDRMODE6502::ZPY => self.zpy(),
      ADDRMODE6502::REL => self.rel(),
      ADDRMODE6502::ABS => self.abs(),
      ADDRMODE6502::ABX => self.abx(),
      ADDRMODE6502::ABY => self.aby(),
      ADDRMODE6502::IND => self.ind(),
      ADDRMODE6502::IZX => self.izx(),
      ADDRMODE6502::IZY => self.izy(),
    }
  }

  /// Implied
  pub fn imp(&mut self) -> u8 {
    self.fetched = self.acc;
    0
  }

  /// Immediate
  pub fn imm(&mut self) -> u8 {
    self.addr_abs = self.pc;
    self.pc_increment();
    0
  }

  /// Zero Page
  pub fn zp0(&mut self) -> u8 {
    self.addr_abs = self.bus_mut_read_u8(self.pc);
    self.pc_increment();
    //self.addr_abs &= 0x00FF;
    0
  }

  /// Zero Page with X offset
  pub fn zpx(&mut self) -> u8 {
    self.addr_abs = self.bus_mut_read_u8(self.pc).wrapping_add(u16::try_from(self.x).unwrap());
    self.pc_increment();
    //self.addr_abs &= 0x00FF;
    0
  }

  /// Zero Page with Y offset
  pub fn zpy(&mut self) -> u8 {
    self.addr_abs = self.bus_mut_read_u8(self.pc).wrapping_add(u16::try_from(self.y).unwrap());
    self.pc_increment();
//    self.addr_abs &= 0x00FF;
    0
  }

  /// Relative
  pub fn rel(&mut self) -> u8 {
    self.addr_rel = u16::try_from(self.bus_mut_read_u8(self.pc)).unwrap();
    self.pc_increment();
    if (self.addr_rel & 0x80) > 0 {
      self.addr_rel |= 0xFF00;
    }
    0
  }

  /// Absolute
  pub fn abs(&mut self) -> u8 {
    let (lo_byte, hi_byte) = self.read_pc();
    self.addr_abs = (hi_byte << 8) | lo_byte;
    0
  }

  /// Absolute with X offset
  pub fn abx(&mut self) -> u8 {
    let (lo_byte, hi_byte) = self.read_pc();

    self.addr_abs = (hi_byte << 8) | lo_byte;
    self.addr_abs = self.addr_abs.wrapping_add(u16::try_from(self.x).unwrap());
    if (self.addr_abs & 0xFF00) != (hi_byte << 8) {
      1
    } else {
      0
    }
  }

  /// Absolute with Y offset
  pub fn aby(&mut self) -> u8 {
    let (lo_byte, hi_byte) = self.read_pc();

    self.addr_abs = (hi_byte << 8) | lo_byte;
    self.addr_abs = self.addr_abs.wrapping_add(u16::try_from(self.y).unwrap());
    if (self.addr_abs & 0xFF00) != (hi_byte << 8) {
      1
    } else {
      0
    }
  }

  /// Indirect
  pub fn ind(&mut self) -> u8 {
    let (lo_byte, hi_byte) = self.read_pc();

    let byte = (hi_byte << 8) | lo_byte;

    self.addr_abs = if lo_byte == 0x00FF {
      (self.bus_mut_read_u8(byte & 0xFF00) << 8) | self.bus_mut_read_u8(byte)
    } else {
      (self.bus_mut_read_u8(byte + 0x01) << 8) | self.bus_mut_read_u8(byte)
    };

    0
  }

  /// Indirect X
  pub fn izx(&mut self) -> u8 {
    let byte = self.bus_mut_read_u8(self.pc);
    self.pc_increment();

    let x = u16::try_from(self.x).unwrap();
    let lo_byte: u16 = self.bus_mut_read_u8((byte + x) & 0x00FF).try_into().unwrap();
    let hi_byte: u16 = self.bus_mut_read_u8((byte + x + 1) & 0x00FF).try_into().unwrap();
    self.addr_abs = (hi_byte << 8) | lo_byte;

    0
  }

  /// Indirect Y
  pub fn izy(&mut self) -> u8 {
    let byte = self.bus_mut_read_u8(self.pc);
    self.pc_increment();

    let lo_byte: u16 = self.bus_mut_read_u8(byte & 0x00FF).try_into().unwrap();
    let hi_byte: u16 = self.bus_mut_read_u8((byte + 1) & 0x00FF).try_into().unwrap();
    self.addr_abs = (hi_byte << 8) | lo_byte;
    self.addr_abs = self.addr_abs.wrapping_add(u16::try_from(self.y).unwrap());

    if (self.addr_abs & 0xFF00) != (hi_byte << 8) {
      1
    } else {
      0
    }
  }

  fn read_pc(&mut self) -> (u16, u16) {
    let lo_byte = self.bus_mut_read_u8(self.pc);
    self.pc_increment();
    let hi_byte = self.bus_mut_read_u8(self.pc);
    self.pc_increment();
    (lo_byte, hi_byte)
  }

  ///OPCODES
  pub fn op_code_value(&mut self, op_code: OPCODES6502) -> u8 {
    match op_code {
      OPCODES6502::ADC => self.adc(),
      OPCODES6502::AND => self.and(),
      OPCODES6502::ASL => self.asl(),
      OPCODES6502::BCC => self.bcc(),
      OPCODES6502::BCS => self.bcs(),
      OPCODES6502::BEQ => self.beq(),
      OPCODES6502::BIT => self.bit(),
      OPCODES6502::BMI => self.bmi(),
      OPCODES6502::BNE => self.bne(),
      OPCODES6502::BPL => self.bpl(),
      OPCODES6502::BRK => self.brk(),
      OPCODES6502::BVC => self.bvc(),
      OPCODES6502::BVS => self.bvs(),
      OPCODES6502::CLC => self.clc(),
      OPCODES6502::CLD => self.cld(),
      OPCODES6502::CLI => self.cli(),
      OPCODES6502::CLV => self.clv(),
      OPCODES6502::CMP => self.cmp(),
      OPCODES6502::CPX => self.cpx(),
      OPCODES6502::CPY => self.cpy(),
      OPCODES6502::DEC => self.dec(),
      OPCODES6502::DEX => self.dex(),
      OPCODES6502::DEY => self.dey(),
      OPCODES6502::EOR => self.eor(),
      OPCODES6502::INC => self.inc(),
      OPCODES6502::INX => self.inx(),
      OPCODES6502::INY => self.iny(),
      OPCODES6502::JMP => self.jmp(),
      OPCODES6502::JSR => self.jsr(),
      OPCODES6502::LDA => self.lda(),
      OPCODES6502::LDX => self.ldx(),
      OPCODES6502::LDY => self.ldy(),
      OPCODES6502::LSR => self.lsr(),
      OPCODES6502::NOP => self.nop(),
      OPCODES6502::ORA => self.ora(),
      OPCODES6502::PHA => self.pha(),
      OPCODES6502::PHP => self.php(),
      OPCODES6502::PLA => self.pla(),
      OPCODES6502::PLP => self.plp(),
      OPCODES6502::ROL => self.rol(),
      OPCODES6502::ROR => self.ror(),
      OPCODES6502::RTI => self.rti(),
      OPCODES6502::RTS => self.rts(),
      OPCODES6502::SBC => self.sbc(),
      OPCODES6502::SEC => self.sec(),
      OPCODES6502::SED => self.sed(),
      OPCODES6502::SEI => self.sei(),
      OPCODES6502::STA => self.sta(),
      OPCODES6502::STX => self.stx(),
      OPCODES6502::STY => self.sty(),
      OPCODES6502::TAX => self.tax(),
      OPCODES6502::TAY => self.tay(),
      OPCODES6502::TSX => self.tsx(),
      OPCODES6502::TXA => self.txa(),
      OPCODES6502::TXS => self.txs(),
      OPCODES6502::TYA => self.tya(),
      OPCODES6502::XXX => 0,
    }
  }

  /// Add with carry
  pub fn adc(&mut self) -> u8 {
    self.fetch();
    self.temp = u16::try_from(self.acc).unwrap()
      + u16::try_from(self.fetched).unwrap()
      + u16::try_from(self.get_flag(&FLAGS6502::C)).unwrap();

    self.set_flag(&FLAGS6502::C, (self.temp) > 255);
    self.set_flag(&FLAGS6502::Z, self.temp.trailing_zeros() > 7);
    self.set_flag(
      &FLAGS6502::V,
      ((!(u16::try_from(self.acc).unwrap()
        ^ u16::try_from(self.fetched).unwrap())
        & (u16::try_from(self.acc).unwrap() ^ u16::try_from(self.temp).unwrap()))
        & 0x0080)
        > 0x00,
    );
    self.set_flag(&FLAGS6502::N, (self.temp & 0x80) > 0x00);
    self.acc = u8::try_from(self.temp & 0x00FF).unwrap();
    1
  }

  /// Arithmetic shift left
  pub fn asl(&mut self) -> u8 {
    self.fetch();
    self.temp = u16::try_from(self.fetched << 1).unwrap();
    self.set_flag(&FLAGS6502::C, (self.temp & 0xFF00) > 0x00);
    self.set_flag(&FLAGS6502::Z, self.temp.trailing_zeros() > 7);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x80) > 0x00);

    if self.addr_mode() == ADDRMODE6502::IMP {
      self.acc = u8::try_from(self.temp & 0x00FF).unwrap();
    } else {
      self.bus_write_u8(self.addr_abs, u8::try_from(self.temp & 0x00FF).unwrap());
    }
    0
  }

  /// and (with accumulator)
  pub fn and(&mut self) -> u8 {
    self.fetch();
    self.acc &= self.fetched;
    self.set_flag(&FLAGS6502::Z, self.acc == 0x00);
    self.set_flag(&FLAGS6502::N, (self.acc & 0x80) > 0x00);
    1
  }

  /// branch on carry clear
  pub fn bcc(&mut self) -> u8 {
    if !self.get_flag(&FLAGS6502::C) {
      self.cycles_increment();
      self.addr_abs = self.pc.wrapping_add(self.addr_rel);

      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles_increment();
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// branch on carry clear
  pub fn bcs(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::C) {
      self.cycles_increment();
      self.addr_abs = self.pc.wrapping_add(self.addr_rel);
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles_increment();
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// branch on equal
  pub fn beq(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::Z) {
      self.cycles_increment();
      self.addr_abs = self.pc.wrapping_add(self.addr_rel);
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles_increment();
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// Bit test
  pub fn bit(&mut self) -> u8 {
    self.fetch();
    self.temp = u16::try_from(self.acc).unwrap() & u16::try_from(self.fetched).unwrap();
    self.set_flag(&FLAGS6502::Z, self.temp.trailing_zeros() > 7);
    self.set_flag(&FLAGS6502::N, (self.fetched & 0x80) > 0x00);
    self.set_flag(&FLAGS6502::V, (self.fetched & 0x40) > 0x00);
    0
  }

  /// Branch on minus (negative set)
  pub fn bmi(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::N) {
      self.cycles_increment();
      self.addr_abs = self.pc.wrapping_add(self.addr_rel);
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles_increment();
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// Branch on not equal (zero clear)
  pub fn bne(&mut self) -> u8 {
    if !self.get_flag(&FLAGS6502::Z) {
      self.cycles_increment();
      self.addr_abs = self.pc.wrapping_add(self.addr_rel);
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles_increment();
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// Branch on plus (negative clear)
  pub fn bpl(&mut self) -> u8 {
    if !self.get_flag(&FLAGS6502::N) {
      self.cycles_increment();
      self.addr_abs = self.pc.wrapping_add(self.addr_rel);
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles_increment();
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// Break / interrupt
  pub fn brk(&mut self) -> u8 {
    self.pc_increment();

    self.set_flag(&FLAGS6502::I, true);
    self.bus_write_u8(
      self.get_stack_address(),
      ((self.pc >> 8) & 0x00FF).try_into().unwrap());
    self.stack_pointer_decrement();

    self.bus_write_u8(
      self.get_stack_address(),
      u8::try_from(self.pc & 0x00FF).unwrap());
    self.stack_pointer_decrement();

    self.set_flag(&FLAGS6502::B, true);
    self.bus_write_u8(
      self.get_stack_address(),
      self.status_register);
    self.stack_pointer_decrement();
    self.set_flag(&FLAGS6502::B, false);

    self.pc = self.bus_mut_read_u8(0xFFFE) | (self.bus_mut_read_u8(0xFFFF) << 8);
    0
  }

  /// Branch on overflow clear
  pub fn bvc(&mut self) -> u8 {
    if !self.get_flag(&FLAGS6502::V) {
      self.cycles_increment();
      self.addr_abs = self.pc.wrapping_add(self.addr_rel);
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles_increment();
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// Branch on overflow clear
  pub fn bvs(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::V) {
      self.cycles_increment();
      self.addr_abs = self.pc.wrapping_add(self.addr_rel);
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles_increment();
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// Clear carry
  pub fn clc(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::C, false);
    0
  }

  /// Clear decimal
  pub fn cld(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::D, false);
    0
  }

  /// Clear interrupt disable
  pub fn cli(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::I, false);
    0
  }

  /// Clear overflow
  pub fn clv(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::V, false);
    0
  }

  /// Compare with accumulator
  pub fn cmp(&mut self) -> u8 {
    self.fetch();
    self.temp = u16::try_from(self.acc.wrapping_sub(self.fetched)).unwrap();
    self.set_flag(&FLAGS6502::C, self.acc >= self.fetched);
    self.set_flag(&FLAGS6502::Z, self.temp.trailing_zeros() > 7);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x0080) > 0x00);
    1
  }

  /// Compare with X
  pub fn cpx(&mut self) -> u8 {
    self.fetch();
    self.temp = u16::try_from(self.x.wrapping_sub(self.fetched)).unwrap();
    self.set_flag(&FLAGS6502::C, self.x >= self.fetched);
    self.set_flag(&FLAGS6502::Z, self.temp.trailing_zeros() > 7);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x0080) > 0x00);
    0
  }

  /// Compare with Y
  pub fn cpy(&mut self) -> u8 {
    self.fetch();
    self.temp = u16::try_from(self.y.wrapping_sub(self.fetched)).unwrap();
    self.set_flag(&FLAGS6502::C, self.y >= self.fetched);
    self.set_flag(&FLAGS6502::Z, self.temp.trailing_zeros() > 7);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x0080) > 0x00);
    0
  }

  /// Decrement
  pub fn dec(&mut self) -> u8 {
    self.fetch();
    self.temp = u16::try_from(self.fetched).unwrap().wrapping_sub(1);
    self.bus_write_u8(self.addr_abs, u8::try_from(self.temp & 0x00FF).unwrap());
    self.set_flag(&FLAGS6502::Z, self.temp.trailing_zeros() > 7);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x0080) > 0x00);
    0
  }

  /// Decrement X
  pub fn dex(&mut self) -> u8 {
    self.x = self.x.wrapping_sub(1);
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, (self.x & 0x80) > 0x00);
    0
  }

  /// Decrement Y
  pub fn dey(&mut self) -> u8 {
    self.y = self.y.wrapping_sub(1);
    self.set_flag(&FLAGS6502::Z, self.y == 0x00);
    self.set_flag(&FLAGS6502::N, (self.y & 0x80) > 0x00);
    0
  }

  /// Exclusive or with accumulator
  pub fn eor(&mut self) -> u8 {
    self.fetch();
    self.acc ^= self.fetched;
    self.set_flag(&FLAGS6502::Z, self.acc == 0x00);
    self.set_flag(&FLAGS6502::N, (self.acc & 0x80) > 0x00);
    1
  }

  /// Increment
  pub fn inc(&mut self) -> u8 {
    self.fetch();
    self.temp = self.fetched.wrapping_add(1).try_into().unwrap();
    self.bus_write_u8(self.addr_abs, u8::try_from(self.temp & 0x00FF).unwrap());
    self.set_flag(&FLAGS6502::Z, self.temp.trailing_zeros() > 7);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x0080) > 0x00);
    0
  }

  /// Increment X
  pub fn inx(&mut self) -> u8 {
    self.x = self.x.wrapping_add(1);
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, (self.x & 0x80) > 0x00);
    0
  }

  /// Increment Y
  pub fn iny(&mut self) -> u8 {
    self.y = self.y.wrapping_add(1);
    self.set_flag(&FLAGS6502::Z, self.y == 0x00);
    self.set_flag(&FLAGS6502::N, (self.y & 0x80) > 0x00);
    0
  }

  /// Jump
  pub fn jmp(&mut self) -> u8 {
    self.pc = self.addr_abs;
    0
  }

  /// Jump subroutine
  pub fn jsr(&mut self) -> u8 {
    self.pc = self.pc.wrapping_sub(1);
    self.bus_write_u8(
      self.get_stack_address(),
      u8::try_from((self.pc >> 8) & 0x00FF).unwrap());
    self.stack_pointer_decrement();
    self.bus_write_u8(self.get_stack_address(),
                      u8::try_from(self.pc & 0x00FF).unwrap());
    self.stack_pointer_decrement();

    self.pc = self.addr_abs;
    0
  }

  /// Load accumulator
  pub fn lda(&mut self) -> u8 {
    self.fetch();
    self.acc = self.fetched;
    self.set_flag(&FLAGS6502::Z, self.acc == 0x00);
    self.set_flag(&FLAGS6502::N, (self.acc & 0x80) > 0x00);
    1
  }

  /// Load X
  pub fn ldx(&mut self) -> u8 {
    self.fetch();
    self.x = self.fetched;
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, (self.x & 0x80) > 0x00);
    1
  }

  /// Load Y
  pub fn ldy(&mut self) -> u8 {
    self.fetch();
    self.y = self.fetched;
    self.set_flag(&FLAGS6502::Z, self.y == 0x00);
    self.set_flag(&FLAGS6502::N, (self.y & 0x80) > 0x00);
    1
  }

  /// Logical shift right
  pub fn lsr(&mut self) -> u8 {
    self.fetch();
    self.set_flag(&FLAGS6502::C, (self.fetched & 1) > 0x00);
    self.temp = u16::try_from(self.fetched.wrapping_shr(1)).unwrap();
    self.set_flag(&FLAGS6502::Z, self.temp.trailing_zeros() > 7);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x0080) > 0x00);

    if self.addr_mode() == ADDRMODE6502::IMP {
      self.acc = u8::try_from(self.temp & 0x00FF).unwrap();
    } else {
      self.bus_write_u8(self.addr_abs, u8::try_from(self.temp & 0x00FF).unwrap());
    }
    0
  }

  /// No operation
  pub fn nop(&mut self) -> u8 {
    match self.opcode {
      0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => 1,
      _ => 0,
    }
  }

  /// Or with accumulator
  pub fn ora(&mut self) -> u8 {
    self.fetch();
    self.acc |= self.fetched;
    self.set_flag(&FLAGS6502::Z, self.acc == 0x00);
    self.set_flag(&FLAGS6502::N, (self.acc & 0x80) > 0x00);
    1
  }

  /// Push accumulator
  pub fn pha(&mut self) -> u8 {
    self.bus_write_u8(self.get_stack_address(), self.acc);
    self.stack_pointer_decrement();
    0
  }

  /// Push processor status (PR)
  pub fn php(&mut self) -> u8 {
    self.bus_write_u8(
      self.get_stack_address(),
      self.status_register | FLAGS6502::B.value() | FLAGS6502::U.value());
    self.set_flag(&FLAGS6502::B, false);
    self.set_flag(&FLAGS6502::U, false);
    self.stack_pointer_decrement();
    0
  }

  /// Pull accumulator
  pub fn pla(&mut self) -> u8 {
    self.stack_pointer_increment();
    self.acc = self
      .bus_mut_read_u8(self.get_stack_address())
      .try_into()
      .unwrap();
    self.set_flag(&FLAGS6502::Z, self.acc == 0x00);
    self.set_flag(&FLAGS6502::N, (self.acc & 0x80) > 0x00);
    0
  }

  /// Pull processor status (SR)
  pub fn plp(&mut self) -> u8 {
    self.stack_pointer_increment();
    self.status_register = self
        .bus_mut_read_u8(0x0100u16 + u16::try_from(self.stack_pointer).unwrap())
        .try_into()
        .unwrap();
    self.set_flag(&FLAGS6502::U, true);
    0
  }

  /// Rotate left
  pub fn rol(&mut self) -> u8 {
    self.fetch();
    self.temp = (u16::try_from(self.fetched).unwrap() << 1) | u16::try_from(self.get_flag(&FLAGS6502::C)).unwrap();
    self.set_flag(&FLAGS6502::C, (self.temp & 0xFF00) > 0x00);
    self.set_flag(&FLAGS6502::Z, self.temp.trailing_zeros() > 7);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x0080) > 0x00);

    if self.addr_mode() == ADDRMODE6502::IMP {
      self.acc = u8::try_from(self.temp & 0x00FF).unwrap();
    } else {
      self.bus_write_u8(self.addr_abs, u8::try_from(self.temp & 0x00FF).unwrap());
    }
    0
  }

  /// Rotate right
  pub fn ror(&mut self) -> u8 {
    self.fetch();
    self.temp = (u16::try_from(self.get_flag(&FLAGS6502::C)).unwrap() << 7) | (u16::try_from(self.fetched).unwrap() >> 1);
    self.set_flag(&FLAGS6502::C, (self.fetched & 0x01) > 0x00);
    self.set_flag(&FLAGS6502::Z, self.temp.trailing_zeros() > 7);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x0080) > 0x00);

    if self.addr_mode() == ADDRMODE6502::IMP {
      self.acc = u8::try_from(self.temp & 0x00FF).unwrap();
    } else {
      self.bus_write_u8(self.addr_abs, u8::try_from(self.temp & 0x00FF).unwrap());
    }
    0
  }

  fn addr_mode(&mut self) -> ADDRMODE6502 {
    let idx = usize::try_from(self.opcode).unwrap();
    *self.lookup.get_addr_mode(idx)
  }

  /// Return form interrupt
  pub fn rti(&mut self) -> u8 {
    self.stack_pointer_increment();
    self.status_register = u8::try_from(self
      .bus_mut_read_u8(self.get_stack_address())).unwrap();
    self.status_register &= !FLAGS6502::B.value();
    self.status_register &= !FLAGS6502::U.value();

    self.stack_pointer_increment();
    self.pc = self.bus_mut_read_u8(self.get_stack_address());
    self.stack_pointer_increment();
    self.pc |= self.bus_mut_read_u8(self.get_stack_address()).wrapping_shl(8);
    0
  }

  /// Return form subroutine
  pub fn rts(&mut self) -> u8 {
    self.stack_pointer_increment();
    self.pc = self.bus_mut_read_u8(self.get_stack_address());

    self.stack_pointer_increment();
    self.pc |= self.bus_mut_read_u8(self.get_stack_address()).wrapping_shl(8);

    self.pc_increment();
    0
  }

  /// Subtract with carry
  pub fn sbc(&mut self) -> u8 {
    self.fetch();
    let value = u16::try_from(self.fetched).unwrap() ^ 0x00FF;

    self.temp = u16::try_from(self.acc).unwrap()
      + value
      + u16::try_from(self.get_flag(&FLAGS6502::C)).unwrap();

    self.set_flag(&FLAGS6502::C, (self.temp & 0xFF00) > 0x00);
    self.set_flag(&FLAGS6502::Z, self.temp.trailing_zeros() > 7);
    self.set_flag(&FLAGS6502::V, (self.temp ^ u16::try_from(self.acc).unwrap() & (self.temp ^ value) & 0x0080) > 0x00);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x0080) > 0x00);
    self.acc = u8::try_from(self.temp & 0x00FF).unwrap();
    1
  }

  /// Set carry
  pub fn sec(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::C, true);
    0
  }

  /// Set decimal
  pub fn sed(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::D, true);
    0
  }

  /// Set interrupt disable
  pub fn sei(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::I, true);
    0
  }

  /// Store accumulator
  pub fn sta(&mut self) -> u8 {
    self.bus_write_u8(self.addr_abs, self.acc);
    0
  }

  /// Store X
  pub fn stx(&mut self) -> u8 {
    self.bus_write_u8(self.addr_abs, self.x);
    0
  }

  /// Store Y
  pub fn sty(&mut self) -> u8 {
    self.bus_write_u8(self.addr_abs, self.y);
    0
  }

  /// Transfer accumulator to X
  pub fn tax(&mut self) -> u8 {
    self.x = self.acc;
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, (self.x & 0x80) > 0x00);
    0
  }

  /// Transfer accumulator to Y
  pub fn tay(&mut self) -> u8 {
    self.y = self.acc;
    self.set_flag(&FLAGS6502::Z, self.y == 0x00);
    self.set_flag(&FLAGS6502::N, (self.y & 0x80) > 0x00);
    0
  }

  /// Transfer stack pointer to X
  pub fn tsx(&mut self) -> u8 {
    self.x = self.stack_pointer;
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, (self.x & 0x80) > 0x00);
    0
  }

  /// Transfer X to accumulator
  pub fn txa(&mut self) -> u8 {
    self.acc = self.x;
    self.set_flag(&FLAGS6502::Z, self.acc == 0x00);
    self.set_flag(&FLAGS6502::N, (self.acc & 0x80) > 0x00);
    0
  }

  /// Transfer X to stack pointer
  pub fn txs(&mut self) -> u8 {
    self.stack_pointer = self.x;
    0
  }

  /// Transfer Y to accumulator
  pub fn tya(&mut self) -> u8 {
    self.acc = self.y;
    self.set_flag(&FLAGS6502::Z, self.acc == 0x00);
    self.set_flag(&FLAGS6502::N, (self.acc & 0x80) > 0x00);
    0
  }

  pub fn disassemble(&mut self, start: u16, end: u16) -> HashMap<u16, String> {
    let mut addr = start as u32;
    let mut map: HashMap<u16, String> = HashMap::new();

    while addr < end as u32 {
      let line_addr = u16::try_from(addr).unwrap();
      let mut codes = format!("$:{}: ", hex(usize::try_from(addr).unwrap(), 4));
      let opcode = self.bus_read_u8(u16::try_from(addr).unwrap());
      addr += 1;

      let name = self.lookup.get_name(opcode.try_into().unwrap());
      codes = format!("{} {} ", codes, name);

      let addr_mode = *self.lookup.get_addr_mode(opcode.try_into().unwrap());

      match addr_mode {
        ADDRMODE6502::IMP => {
          codes.push_str(" {{IMP}}\t");
        }
        ADDRMODE6502::IMM => {
          let value = self.bus_read_u8(addr.try_into().unwrap());
          addr += 1;
          codes.push_str(&format!("${} {{IMM}}\t", hex(usize::from(value), 2)));
        }
        ADDRMODE6502::ZP0 => {
          let lo_byte = self.bus_read_u8(u16::try_from(addr).unwrap());
          addr += 1;
          codes.push_str(&format!("${} {{ZP0}}\t", hex(usize::from(lo_byte), 2)));
        }
        ADDRMODE6502::ZPX => {
          let lo_byte = self.bus_read_u8(addr.try_into().unwrap());
          addr += 1;
          codes.push_str(&format!("${} {{ZPX}}\t", hex(usize::from(lo_byte), 2)));
        }
        ADDRMODE6502::ZPY => {
          let lo_byte = self.bus_read_u8(addr.try_into().unwrap());
          addr += 1;
          codes.push_str(&format!("${} {{ZPY}}\t", hex(usize::from(lo_byte), 2)));
        }
        ADDRMODE6502::REL => {
          let value = self.bus_read_u8(addr.try_into().unwrap());
          addr += 1;
          codes.push_str(&format!(
            "${} [${}] {{REL}}\t",
            hex(usize::from(value), 2),
            hex((addr.wrapping_add(value.into())).try_into().unwrap(), 4)
          ));
        }
        ADDRMODE6502::ABS => {
          let (lo_byte, hi_byte) = self.extract_addr_16(addr);
          codes.push_str(&format!(
            "${} {{ABS}}\t",
            hex(usize::from(hi_byte.wrapping_shl(8) | lo_byte), 4)
          ));
        }
        ADDRMODE6502::ABX => {
          let (lo_byte, hi_byte) = self.extract_addr_16(addr);
          codes.push_str(&format!(
            "${} X {{ABX}}\t",
            hex(usize::from(hi_byte.wrapping_shl(8) | lo_byte), 4)
          ));
        }
        ADDRMODE6502::ABY => {
          let (lo_byte, hi_byte) = self.extract_addr_16(addr);
          codes.push_str(&format!(
            "${}, Y {{ABY}}\t",
            hex(usize::from(hi_byte.wrapping_shl(8) | lo_byte), 4)
          ));
        }
        ADDRMODE6502::IND => {
          let (lo_byte, hi_byte) = self.extract_addr_16(addr);
          codes.push_str(&format!(
            "(${}) {{IND}}\t",
            hex(usize::from(hi_byte.wrapping_shl(8) | lo_byte), 4)
          ));
        }
        ADDRMODE6502::IZX => {
          let lo_byte = self.bus_read_u8(addr.try_into().unwrap());
          addr += 1;
          codes.push_str(&format!("${} {{IZX}}\t", hex(usize::from(lo_byte), 2)));
        }
        ADDRMODE6502::IZY => {
          let lo_byte = self.bus_read_u8(addr.try_into().unwrap());
          addr += 1;
          codes.push_str(&format!("${} {{IZY}}\t", hex(usize::from(lo_byte), 2)));
        }
      }

      map.insert(line_addr, codes);
    }
    map
  }

  fn extract_addr_16(&mut self, mut addr: u32) -> (u16, u16) {
    let lo_byte = self.bus_read_u8(addr.try_into().unwrap());
    addr += 1;
    let hi_byte = self.bus_read_u8(addr.try_into().unwrap());
    (lo_byte, hi_byte)
  }
}

pub fn hex(num: usize, len: usize) -> String {
  match len {
    2 => format!("{:0>2X}", num),
    4 => format!("{:0>4X}", num),
    _ => panic!("Unknown length"),
  }
}

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use crate::bus::Bus;
use crate::cpu::instruction_table::{ADDRMODE6502, FLAGS6502, LookUpTable, OPCODES6502};

pub mod instruction_table;

pub struct Cpu<'a> {
  pub bus: &'a mut Bus,
  pub pc: u16,
  pub a: u8,
  pub x: u8,
  pub y: u8,
  pub status_register: u8,
  pub stack_pointer: u8,
  pub fetched: u8,
  pub temp: u16,
  pub addr_abs: u16,
  pub addr_rel: u16,
  pub opcode: u8,
  pub cycles: u8,
  pub clock_count: usize,
  lookup: LookUpTable,
}

impl<'a> Cpu<'a> {
  pub fn new(bus: &mut Bus) -> Cpu {
    let lookup = LookUpTable::new();

    Cpu {
      bus,
      pc: 0,
      a: 0,
      x: 0,
      y: 0,
      fetched: 0x0,
      temp: 0,
      addr_abs: 0,
      addr_rel: 0,
      opcode: 0x0,
      stack_pointer: 0x0,
      status_register: 0,
      cycles: 0,
      clock_count: 0,
      lookup,
    }
  }

  pub fn complete(&self) -> bool {
    self.cycles == 0
  }

  fn set_flag(&mut self, flag: &FLAGS6502, val: bool) {
    let f = flag.value();
    if val {
      self.status_register |= f;
    } else {
      self.status_register &= !f;
    }
  }

  fn get_flag(&self, flag: &FLAGS6502) -> u8 {
    let f = flag.value();
    if (self.status_register & f) > 0 {
      1
    } else {
      0
    }
  }

  pub fn clock(&mut self) {
    if self.cycles == 0 {
      self.opcode = self.bus.read_u8(self.pc).try_into().unwrap();

      self.set_flag(&FLAGS6502::U, true);

      self.pc = self.pc.wrapping_add(1);

      let idx = usize::try_from(self.opcode).unwrap_or(0);
      let addr_mode = self.lookup.get_addr_mode(idx).clone();

      let operate = self.lookup.get_operate(idx).clone();

      self.cycles += self.addr_mode_value(addr_mode) &  self.op_code_value(operate);

      self.set_flag(&FLAGS6502::U, true);

      self.clock_count = self.clock_count.wrapping_add(1);

      self.cycles = self.cycles.wrapping_sub(1);
    }
  }

  pub fn fetch(&mut self) -> u8 {
    let idx = usize::try_from(self.opcode).unwrap_or(0);
    let addr_mode = self.lookup.get_addr_mode(idx).clone();
    if addr_mode == ADDRMODE6502::IMP {
      let addr = u16::try_from(self.addr_mode_value(addr_mode.clone())).unwrap();
      self.fetched = self.bus.read_u8(addr).try_into().unwrap();
    }
    self.fetched
  }

  pub fn reset(&mut self) {
    let address_abs = 0xFFFC;

    let lo_byte = self.bus.read_u8(address_abs + 0);
    let hi_byte = self.bus.read_u8(address_abs + 1);

    self.pc = (hi_byte.wrapping_shl(8)) | lo_byte;
    self.a = 0;
    self.x = 0;
    self.y = 0;
    self.fetched = 0x00;
    self.status_register = 0;

    self.cycles = 0;
  }

  pub fn irq(&mut self) {
    if self.get_flag(&FLAGS6502::I) == 0x00 {
      self.bus.write_u8(
        0x0100 + u16::try_from(self.stack_pointer).unwrap(),
        u8::try_from((self.pc.wrapping_shr(8)) & 0x00FF).unwrap(),
      );
      self.stack_pointer = self.stack_pointer.wrapping_sub(1);
      self.bus.write_u8(
        0x0100 + u16::try_from(self.stack_pointer).unwrap(),
        u8::try_from(self.pc & 0x00FF).unwrap(),
      );
      self.stack_pointer = self.stack_pointer.wrapping_sub(1);

      self.set_flag(&FLAGS6502::B, false);
      self.set_flag(&FLAGS6502::U, true);
      self.set_flag(&FLAGS6502::I, true);

      self.bus.write_u8(
        0x100 + u16::try_from(self.stack_pointer).unwrap(),
        self.status_register,
      );
      self.stack_pointer = self.stack_pointer.wrapping_sub(1);

      let address_abs = 0xFFFE;
      let lo_byte = self.bus.read_u8(address_abs);
      let hi_byte = self.bus.read_u8(address_abs + 1);
      self.pc = u16::try_from((hi_byte.wrapping_shl(8)) | lo_byte).unwrap();

      self.cycles = 7;
    }
  }

  pub fn nmi(&mut self) {
    self.bus.write_u8(
      0x01000 + u16::try_from(self.stack_pointer).unwrap(),
      u8::try_from((self.pc.wrapping_shr(8)) & 0x00FF).unwrap(),
    );
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    self.bus.write_u8(
      0x01000 + u16::try_from(self.stack_pointer).unwrap(),
      u8::try_from(self.pc & 0x00FF).unwrap(),
    );
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);

    self.set_flag(&FLAGS6502::B, false);
    self.set_flag(&FLAGS6502::U, true);
    self.set_flag(&FLAGS6502::I, true);
    self.bus.write_u8(
      0x100 + u16::try_from(self.stack_pointer).unwrap(),
      self.status_register,
    );
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);

    let address_abs = 0xFFFA;
    let lo_byte = self.bus.read_u8(address_abs);
    let hi_byte = self.bus.read_u8(address_abs + 1);
    self.pc = (hi_byte.wrapping_shl(8)) | lo_byte;

    self.cycles = 8;
  }

  fn fetch_data(&mut self) -> u8 {
    let idx = usize::try_from(self.opcode).unwrap_or(0);
    let addr_mode = self.lookup.get_addr_mode(idx).clone();
    if addr_mode == ADDRMODE6502::IMP {
      self.fetched = self.bus.read_u8(self.addr_abs).try_into().unwrap();
    }
    self.fetched
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

  pub fn imp(&mut self) -> u8 {
    self.fetched = self.a;
    0
  }

  pub fn imm(&mut self) -> u8 {
    self.addr_abs = self.pc;
    self.pc = self.pc.wrapping_add(1);
    0
  }

  pub fn zp0(&mut self) -> u8 {
    self.addr_abs = self.bus.read_u8(self.pc);
    self.pc = self.pc.wrapping_add(1);
    self.addr_abs &= 0x00FF;
    0
  }

  pub fn zpx(&mut self) -> u8 {
    let x: u16 = self.x.try_into().unwrap();
    self.addr_abs = self.bus.read_u8(self.pc) + x;
    self.pc = self.pc.wrapping_add(1);
    self.addr_abs &= 0x00FF;
    0
  }

  pub fn zpy(&mut self) -> u8 {
    let y: u16 = self.y.try_into().unwrap();
    self.addr_abs = self.bus.read_u8(self.pc) + y;
    self.pc = self.pc.wrapping_add(1);
    self.addr_abs &= 0x00FF;
    0
  }

  pub fn rel(&mut self) -> u8 {
    self.addr_rel = u16::try_from(self.bus.read_u8(self.pc)).unwrap();
    self.pc = self.pc.wrapping_add(1);
    if self.addr_rel & 0x80 != 0 {
      self.addr_rel |= 0xFF00;
    }
    0
  }

  pub fn abs(&mut self) -> u8 {
    let (lo_byte, hi_byte) = self.read_pc();
    self.addr_abs = u16::try_from((hi_byte.wrapping_shl(8)) | lo_byte).unwrap();
    0
  }

  fn read_pc(&mut self) -> (u16, u16) {
    let lo_byte = self.bus.read_u8(self.pc);
    self.pc = self.pc.wrapping_add(1);
    let hi_byte = self.bus.read_u8(self.pc);
    self.pc  = self.pc.wrapping_add(1);
    (lo_byte, hi_byte)
  }

  pub fn abx(&mut self) -> u8 {
    let (lo_byte, hi_byte) = self.read_pc();

    self.addr_abs = u16::try_from((hi_byte.wrapping_shl(8)) | lo_byte).unwrap();
    self.addr_abs += self.x as u16;
    if (self.addr_abs & 0xFF00) != u16::try_from(hi_byte.wrapping_shl(8)).unwrap() {
      1
    } else {
      0
    }
  }

  pub fn aby(&mut self) -> u8 {
    let (lo_byte, hi_byte) = self.read_pc();

    self.addr_abs = u16::try_from((hi_byte.wrapping_shl(8)) | lo_byte).unwrap();
    self.addr_abs += u16::try_from(self.y).unwrap();
    if (self.addr_abs & 0xFF00) != u16::try_from(hi_byte.wrapping_shl(8)).unwrap() {
      1
    } else {
      0
    }
  }

  pub fn ind(&mut self) -> u8 {
    let (lo_byte, hi_byte) = self.read_pc();

    let byte = (hi_byte.wrapping_shl(8)) | lo_byte;

    if lo_byte == 0x00FF {
      self.addr_abs = ((self.bus.read_u8(byte & 0xFF00).wrapping_shl(8)) | self.bus.read_u8(byte)).try_into().unwrap();
    } else {
      self.addr_abs = (self.bus.read_u8((byte + 1).wrapping_shl(8)) | self.bus.read_u8(byte)).try_into().unwrap()
    }

    0
  }

  pub fn izx(&mut self) -> u8 {
    let byte = self.bus.read_u8(self.pc);
    self.pc = self.pc.wrapping_add(1);

    let x: u16 = self.x.try_into().unwrap();
    let lo_byte: u16 = self.bus.read_u8((byte + x) & 0x00FF).try_into().unwrap();
    let hi_byte: u16 = self.bus.read_u8((byte + x + 1) & 0x00FF).try_into().unwrap();
    self.addr_abs = (hi_byte.wrapping_shl(8)) | lo_byte;

    0
  }

  pub fn izy(&mut self) -> u8 {
    let byte = self.bus.read_u8(self.pc);
    self.pc = self.pc.wrapping_add(1);

    let y: u16 = self.y.try_into().unwrap();
    let lo_byte: u16 = self.bus.read_u8((byte + y) & 0x00FF).try_into().unwrap();
    let hi_byte: u16 = self.bus.read_u8((byte + y + 1) & 0x00FF).try_into().unwrap();
    self.addr_abs = ((hi_byte.wrapping_shl(8)) | lo_byte).try_into().unwrap();
    self.addr_abs += u16::try_from(self.y).unwrap();


    if (self.addr_abs & 0xFF00) != u16::try_from(hi_byte.wrapping_shl(8)).unwrap() {
      1
    } else {
      0
    }
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
      OPCODES6502::XXX => 0u8,
    }
  }

  /// Add with carry
  pub fn adc(&mut self) -> u8 {
    self.fetch();
    self.temp = u16::try_from(self.a + self.fetched + self.get_flag(&FLAGS6502::C)).unwrap();

    self.set_flag(&FLAGS6502::C, self.temp > 255);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) != 0x00);
    self.set_flag(&FLAGS6502::V, (!(self.a ^ self.fetched) & u8::try_from(u16::try_from(self.a).unwrap() ^ self.temp).unwrap()) & 0x0080 != 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x80 != 0x00);
    self.a = (self.temp & 0x00FF) as u8;
    1
  }

  /// Arithmetic shift left
  pub fn asl(&mut self) -> u8 {
    self.fetch();
    self.temp = (self.fetched as u16).wrapping_shl(1);
    self.set_flag(&FLAGS6502::C, (self.temp & 0xFF00) > 0x00);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0xFF00) == 0x00);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x80) != 0x00);

    let idx = usize::try_from(self.opcode).unwrap_or(0);
    let addr_mode = self.lookup.get_addr_mode(idx).clone();

    if addr_mode == ADDRMODE6502::IMP {
      self.a = (self.temp & 0x00FF).try_into().unwrap();
    } else {
      self.bus.write_u8(self.addr_abs, (self.temp & 0x00FF) as u8);
    }
    0
  }

  /// and (with accumulator)
  pub fn and(&mut self) -> u8 {
    self.fetch();
    self.a = self.a & self.fetched;
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x80 != 0x00);
    1
  }

  /// branch on carry clear
  pub fn bcc(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::C) == 0x00 {
      self.cycles = self.cycles.wrapping_add(1);
      self.addr_abs = self.pc + self.addr_rel;

      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles = self.cycles.wrapping_add(1);
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// branch on carry clear
  pub fn bcs(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::C) == 0x01 {
      self.cycles = self.cycles.wrapping_add(1);
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles = self.cycles.wrapping_add(1);
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// branch on equal
  pub fn beq(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::Z) == 0x01 {
      self.cycles = self.cycles.wrapping_add(1);
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles = self.cycles.wrapping_add(1);
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// Bit test
  pub fn bit(&mut self) -> u8 {
    self.fetch();
    self.temp = (self.a & self.fetched) as u16;
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) == 0x00);
    self.set_flag(&FLAGS6502::N, self.fetched & (1 << 7) != 0x00);
    self.set_flag(&FLAGS6502::V, self.fetched & (1 << 6) != 0x00);
    0
  }

  /// Branch on minus (negative set)
  pub fn bmi(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::N) == 0x01 {
      self.cycles = self.cycles.wrapping_add(1);
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles = self.cycles.wrapping_add(1);
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// Branch on not equal (zero clear)
  pub fn bne(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::Z) == 0x00 {
      self.cycles = self.cycles.wrapping_add(1);
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles = self.cycles.wrapping_add(1);
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// Branch on plus (negative clear)
  pub fn bpl(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::N) == 0x00 {
      self.cycles = self.cycles.wrapping_add(1);
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles = self.cycles.wrapping_add(1);
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// Break / interrupt
  pub fn brk(&mut self) -> u8 {
    self.pc = self.pc.wrapping_add(1);

    self.set_flag(&FLAGS6502::I, true);
    self.bus.write_u8(0x0100 + u16::try_from(self.stack_pointer).unwrap(), (self.pc.wrapping_shr(8) & 0x00FF).try_into().unwrap());
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    self.bus.write_u8(0x0100 + u16::try_from(self.stack_pointer).unwrap(), u8::try_from(self.pc).unwrap() & 0x00FF);
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);

    self.set_flag(&FLAGS6502::B, true);
    self.bus.write_u8(0x0100 + u16::try_from(self.stack_pointer).unwrap(), self.status_register);
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    self.set_flag(&FLAGS6502::B, false);

    self.pc = u16::try_from(self.bus.read_u8(0xFFFE)).unwrap() | (u16::try_from(self.bus.read_u8(0xFFFF)).unwrap()).wrapping_shl(8);
    0
  }

  /// Branch on overflow clear
  pub fn bvc(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::V) == 0 {
      self.cycles = self.cycles.wrapping_add(1);
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles = self.cycles.wrapping_add(1);
      }
      self.pc = self.addr_abs;
    }
    0
  }

  /// Branch on overflow clear
  pub fn bvs(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::V) == 1 {
      self.cycles = self.cycles.wrapping_add(1);
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles = self.cycles.wrapping_add(1);
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
    self.temp = self.a as u16 - self.fetched as u16;
    self.set_flag(&FLAGS6502::C, self.a >= self.fetched);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 != 0x00);
    1
  }

  /// Compare with X
  pub fn cpx(&mut self) -> u8 {
    self.fetch();
    self.temp = self.x as u16 - self.fetched as u16;
    self.set_flag(&FLAGS6502::C, self.x >= self.fetched);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 != 0x00);
    0
  }

  /// Compare with Y
  pub fn cpy(&mut self) -> u8 {
    self.fetch();
    self.temp = self.y as u16 - self.fetched as u16;
    self.set_flag(&FLAGS6502::C, self.y >= self.fetched);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 != 0x00);
    0
  }

  /// Decrement
  pub fn dec(&mut self) -> u8 {
    self.fetch();
    self.temp = self.fetched as u16 - 1;
    self.bus.write_u8(self.addr_abs, self.temp as u8 & 0x00FF);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 != 0x00);
    0
  }

  /// Decrement X
  pub fn dex(&mut self) -> u8 {
    self.x = self.x.wrapping_sub(1);
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, self.x & 0x80 != 0x00);
    0
  }

  /// Decrement Y
  pub fn dey(&mut self) -> u8 {
    self.y = self.y.wrapping_sub(1);
    self.set_flag(&FLAGS6502::Z, self.y == 0x00);
    self.set_flag(&FLAGS6502::N, self.y & 0x80 != 0x00);
    0
  }

  /// Exclusive or with accumulator
  pub fn eor(&mut self) -> u8 {
    self.fetch();
    self.a ^= self.fetched;
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x80 != 0x00);
    1
  }

  /// Increment
  pub fn inc(&mut self) -> u8 {
    self.fetch();
    self.temp = self.fetched.wrapping_add(1).try_into().unwrap();
    self.bus.write_u8(self.addr_abs, self.temp as u8 & 0x00FF);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) == 0x00);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x0080) != 0x00);
    0
  }

  /// Increment X
  pub fn inx(&mut self) -> u8 {
    self.x = self.x.wrapping_add(1);
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, self.x & 0x80 != 0x00);
    0
  }

  /// Increment Y
  pub fn iny(&mut self) -> u8 {
    self.y = self.y.wrapping_add(1);
    self.set_flag(&FLAGS6502::Z, self.y == 0x00);
    self.set_flag(&FLAGS6502::N, self.y & 0x80 != 0x00);
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
    self.bus.write_u8(0x0100 & u16::try_from(self.stack_pointer).unwrap(), (self.pc.wrapping_shr(8)) as u8 & 0x00FF);
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    self.bus.write_u8(0x0100 & u16::try_from(self.stack_pointer).unwrap(), self.pc as u8 & 0x00FF);
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);

    self.pc = self.addr_abs;
    0
  }

  /// Load accumulator
  pub fn lda(&mut self) -> u8 {
    self.fetch();
    self.a = self.fetched;
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x80 != 0x00);
    1
  }

  /// Load X
  pub fn ldx(&mut self) -> u8 {
    self.fetch();
    self.x = self.fetched;
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, self.x & 0x80 > 0x00);
    1
  }

  /// Load Y
  pub fn ldy(&mut self) -> u8 {
    self.fetch();
    self.y = self.fetched;
    self.set_flag(&FLAGS6502::Z, self.y == 0x00);
    self.set_flag(&FLAGS6502::N, self.y & 0x80 != 0x00);
    1
  }

  /// Logical shift right
  pub fn lsr(&mut self) -> u8 {
    self.fetch();
    self.set_flag(&FLAGS6502::C, self.fetched & 0x0001 != 0x00);
    self.temp = self.fetched as u16 >> 1;
    self.set_flag(&FLAGS6502::Z, self.temp & 0x00FF == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 != 0x00);

    let idx = usize::try_from(self.opcode).unwrap_or(0);
    let addr_mode = self.lookup.get_addr_mode(idx).clone();

    if addr_mode == ADDRMODE6502::IMP {
      self.a = self.temp as u8 & 0x00FF;
    } else {
      self.bus.write_u8(self.addr_abs, self.temp as u8 & 0x00FF);
    }
    0
  }

  /// No operation
  pub fn nop(&mut self) -> u8 {
    match self.opcode {
      0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => 1,
      _ => 0
    }
  }

  /// Or with accumulator
  pub fn ora(&mut self) -> u8 {
    self.fetch();
    self.a = self.a | self.fetched;
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x0080 != 0x00);
    1
  }

  /// Push accumulator
  pub fn pha(&mut self) -> u8 {
    self.bus.write_u8(0x0100 + u16::try_from(self.stack_pointer).unwrap(), self.a);
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    0
  }

  /// Push processor status (PR)
  pub fn php(&mut self) -> u8 {
    self.bus.write_u8(0x0100 + u16::try_from(self.stack_pointer).unwrap(), self.status_register | FLAGS6502::B.value() | FLAGS6502::U.value());
    self.set_flag(&FLAGS6502::B, false);
    self.set_flag(&FLAGS6502::U, false);
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    0
  }

  /// Pull accumulator
  pub fn pla(&mut self) -> u8 {
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
    self.a = self.bus.read_u8(0x0100 + u16::try_from(self.stack_pointer).unwrap()).try_into().unwrap();
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x80 != 0x00);
    0
  }

  /// Pull processor status (SR)
  pub fn plp(&mut self) -> u8 {
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
    self.status_register = self.bus.read_u8(0x0100 + u16::try_from(self.stack_pointer).unwrap()).try_into().unwrap();
    self.set_flag(&FLAGS6502::U, true);
    0
  }

  /// Rotate left
  pub fn rol(&mut self) -> u8 {
    self.fetch();
    self.temp = (self.fetched.wrapping_shl(1)) as u16 | self.get_flag(&FLAGS6502::C) as u16;
    self.set_flag(&FLAGS6502::C, self.temp & 0xFF00 != 0x00);
    self.set_flag(&FLAGS6502::Z, self.temp & 0x00FF == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 != 0x00);

    let idx = usize::try_from(self.opcode).unwrap_or(0);
    let addr_mode = self.lookup.get_addr_mode(idx).clone();
    if addr_mode == ADDRMODE6502::IMP {
      self.a = self.temp as u8 & 0x00FF;
    } else {
      self.bus.write_u8(self.addr_abs, self.temp as u8 & 0x00FF);
    }
    0
  }

  /// Rotate right
  pub fn ror(&mut self) -> u8 {
    self.fetch();
    self.temp = self.get_flag(&FLAGS6502::C) as u16 | (self.fetched.wrapping_shl(1)) as u16;
    self.set_flag(&FLAGS6502::C, self.fetched & 0x01 != 0x00);
    self.set_flag(&FLAGS6502::Z, self.temp & 0x00FF == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 != 0x00);

    let idx = usize::try_from(self.opcode).unwrap_or(0);
    let addr_mode = self.lookup.get_addr_mode(idx).clone();
    if addr_mode == ADDRMODE6502::IMP {
      self.a = self.temp as u8 & 0x00FF;
    } else {
      self.bus.write_u8(self.addr_abs, self.temp as u8 & 0x00FF);
    }
    0
  }

  /// Return form interrupt
  pub fn rti(&mut self) -> u8 {
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
    self.status_register = self.bus.read_u8(0x0100 + u16::try_from(self.stack_pointer).unwrap()).try_into().unwrap();
    self.status_register &= !FLAGS6502::B.value();
    self.status_register &= !FLAGS6502::U.value();

    self.stack_pointer = self.stack_pointer.wrapping_add(1);
    self.pc = self.bus.read_u8(0x0100 + u16::try_from(self.stack_pointer).unwrap()).try_into().unwrap();
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
    self.pc |= (self.bus.read_u8(0x0100 + u16::try_from(self.stack_pointer).unwrap()) as u16).wrapping_shl(8);
    0
  }

  /// Return form subroutine
  pub fn rts(&mut self) -> u8 {
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
    self.pc = self.bus.read_u8(0x0100 + u16::try_from(self.stack_pointer).unwrap());
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
    self.pc |= self.bus.read_u8(0x0100 + u16::try_from(self.stack_pointer).unwrap()).wrapping_shl(8);

    self.pc = self.pc.wrapping_add(1);
    0
  }

  /// Subtract with carry
  pub fn sbc(&mut self) -> u8 {
    self.fetch();
    let value = self.fetched as u16 ^ 0x00FF;

    self.temp = (self.a as u16 + value + u16::try_from(self.get_flag(&FLAGS6502::C)).unwrap());
    self.set_flag(&FLAGS6502::C, self.temp & 0xFF00 != 0x00);
    self.set_flag(&FLAGS6502::Z, self.temp & 0x00FF == 0);
    self.set_flag(&FLAGS6502::V, (self.temp ^ self.a as u16 & (self.temp ^ value as u16) & 0x0080) != 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 != 0x00);
    self.a = self.temp as u8 & 0x00FF;
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
    self.bus.write_u8(self.addr_abs, self.a);
    0
  }

  /// Store X
  pub fn stx(&mut self) -> u8 {
    self.bus.write_u8(self.addr_abs, self.x);
    0
  }

  /// Store Y
  pub fn sty(&mut self) -> u8 {
    self.bus.write_u8(self.addr_abs, self.y);
    0
  }

  /// Transfer accumulator to X
  pub fn tax(&mut self) -> u8 {
    self.x = self.a;
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, self.x & 0x80 != 0x00);
    0
  }

  /// Transfer accumulator to Y
  pub fn tay(&mut self) -> u8 {
    self.y = self.a;
    self.set_flag(&FLAGS6502::Z, self.y == 0x00);
    self.set_flag(&FLAGS6502::N, self.y & 0x80 != 0x00);
    0
  }

  /// Transfer stack pointer to X
  pub fn tsx(&mut self) -> u8 {
    self.x = self.stack_pointer;
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, self.x & 0x80 != 0x00);
    0
  }

  /// Transfer X to accumulator
  pub fn txa(&mut self) -> u8 {
    self.a = self.x;
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x80 != 0x00);
    0
  }

  /// Transfer X to stack pointer
  pub fn txs(&mut self) -> u8 {
    self.stack_pointer = self.x;
    0
  }

  /// Transfer Y to accumulator
  pub fn tya(&mut self) -> u8 {
    self.a = self.y;
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x80 != 0x00);
    0
  }

  pub fn disassemble(&self, start: u16, end: u16) -> HashMap<u16, String> {
    let mut addr = start as u32;
    let mut line_addr = 0;
    let mut map: HashMap<u16, String> = HashMap::new();

    while addr <= end as u32 {
      line_addr = addr as u16;

      let mut codes = format!("$:{}: ", hex(addr as usize, 4));
      let opcode = self.bus.read_u8(addr as u16);
      addr += 1;

      let name = self.lookup.get_name(opcode.try_into().unwrap());
      codes = format!("{} {} ", codes, name);

      let addr_mode = self.lookup.get_addr_mode(opcode.try_into().unwrap()).clone();

      if opcode != 0 {
        println!("{} -> {} -> {:?}", addr, opcode, addr_mode);
      }

      match addr_mode {
        ADDRMODE6502::IMP => {
          codes = format!("{} {{IMP}} ", codes);
        }
        ADDRMODE6502::IMM => {
          let value = self.bus.read_u8(addr as u16);
          addr += 1;
          codes.push_str(&format!("${} {{IMM}}", hex(value as usize, 2)));
        }
        ADDRMODE6502::ZP0 => {
          let lo_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          codes.push_str( &format!("${} {{ZP0}} ", hex(lo_byte as usize, 2)));
        }
        ADDRMODE6502::ZPX => {
          let lo_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          codes.push_str( &format!("${} {{ZPX}} ", hex(lo_byte as usize, 2)));
        }
        ADDRMODE6502::ZPY => {
          let lo_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          codes.push_str( &format!("${} {{ZPY}} ", hex(lo_byte as usize, 2)));
        }
        ADDRMODE6502::REL => {
          let value = self.bus.read_u8(addr as u16);
          addr += 1;
          codes.push_str( &format!("${} [${}] {{REL}} ", hex(value as usize, 2), hex((addr+value as u32).try_into().unwrap(), 4)));
        }
        ADDRMODE6502::ABS => {
          let lo_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          let hi_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          codes.push_str( &format!("${} {{ABS}} ", hex(usize::from(hi_byte.wrapping_shl(8) | lo_byte), 4)));
        }
        ADDRMODE6502::ABX => {
          let lo_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          let hi_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          codes.push_str( &format!("${} X {{ABX}} ", hex(usize::from(hi_byte.wrapping_shl(8) | lo_byte), 4)));
        }
        ADDRMODE6502::ABY => {
          let lo_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          let hi_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          codes.push_str( &format!("${}, Y {{ABY}} ", hex(usize::from(hi_byte.wrapping_shl(8) | lo_byte), 4)));
        }
        ADDRMODE6502::IND => {
          let lo_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          let hi_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          codes.push_str( &format!("(${}) {{IND}} ", hex(usize::from(hi_byte.wrapping_shl(8) | lo_byte), 4)));
        }
        ADDRMODE6502::IZX => {
          let lo_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          codes.push_str( &format!("${} {{IZX}} ", hex(lo_byte as usize, 2)));
        }
        ADDRMODE6502::IZY => {
          let lo_byte = self.bus.read_u8(addr as u16);
          addr += 1;
          codes.push_str( &format!("${} {{IZY}} ", hex(lo_byte as usize, 2)));
        }
      }

      map.insert(line_addr, codes);
    }
    map
  }
}

pub fn hex(num: usize, len: usize) -> String {
  match len {
    2 => format!("{:0>2X}", num),
    4 => format!("{:0>4X}", num),
    _ => panic!("Unknown length")
  }
}

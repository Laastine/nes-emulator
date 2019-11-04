use std::collections::HashMap;
use std::convert::TryFrom;

use crate::bus::Bus;
use crate::cpu;
use crate::cpu::instruction_table::{ADDRMODE6502, FLAGS6502, LookUpTable, OPCODES6502};

mod instruction_table;

pub struct Cpu<'a> {
  bus: &'a mut Bus,
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
      let op_code = self.bus.read_u8(self.pc);

      self.set_flag(&FLAGS6502::U, true);

      self.pc += 1;

      let idx = usize::try_from(op_code).unwrap_or(0);

      let instruction = &self.lookup.instructions[idx];
      let mut cycles = instruction.cycles;

      let addr_mode = self.lookup.get_addr_mode(idx).clone();
      let op_code = self.lookup.get_operate(idx).clone();

      let additional_cycle_1 = self.addr_mode_value(addr_mode);
      let additional_cycle_2 = self.op_code_value(op_code);

      self.cycles += additional_cycle_1 & additional_cycle_2;

      self.set_flag(&FLAGS6502::U, true);

      self.clock_count += 1;

      self.cycles -= 1;
    }
  }

  pub fn fetch(&mut self) -> u8 {
    let idx = usize::try_from(self.opcode).unwrap_or(0);
    let addr_mode = self.lookup.get_addr_mode(idx).clone();
    if addr_mode == ADDRMODE6502::IMP {
      let addr = self.addr_mode_value(addr_mode.clone()) as u16;
      self.fetched = self.bus.read_u8(addr);
    }
    self.fetched
  }

  pub fn reset(&mut self) {
    let address_abs = 0xFFFC;

    self.pc = self.read_u16(address_abs);
    self.a = 0;
    self.x = 0;
    self.y = 0;
    self.fetched = 0x00;
    self.status_register = 0;

    self.cycles = 0;
  }

  pub fn irq(&mut self) {
    if self.status_register == 0u8 {
      self.bus.write_u8(
        0x0100 + self.stack_pointer as u16,
        ((self.pc >> 8) & 0x00FF) as u8,
      );
      self.stack_pointer -= 1;
      self.bus.write_u8(
        0x0100 + self.stack_pointer as u16,
        (self.pc & 0x00FF) as u8,
      );
      self.stack_pointer -= 1;

      self.set_flag(&FLAGS6502::B, false);
      self.set_flag(&FLAGS6502::U, true);
      self.set_flag(&FLAGS6502::I, true);

      self.bus.write_u8(
        0x100 + self.stack_pointer as u16,
        self.status_register,
      );
      self.stack_pointer -= 1;

      let address_abs = 0xFFFE;
      let lo_byte = self.bus.read_u8(address_abs);
      let hi_byte = self.bus.read_u8(address_abs + 1);
      self.pc = ((hi_byte << 8) | lo_byte) as u16;

      self.cycles = 7;
    }
  }

  pub fn nmi(&mut self) {
    self.bus.write_u8(
      0x01000 + self.stack_pointer as u16,
      ((self.pc >> 8) & 0x00FF) as u8,
    );
    self.stack_pointer -= 1;
    self.bus.write_u8(
      0x01000 + self.stack_pointer as u16,
      (self.pc & 0x00FF) as u8,
    );
    self.stack_pointer -= 1;

    self.set_flag(&FLAGS6502::B, false);
    self.set_flag(&FLAGS6502::U, true);
    self.set_flag(&FLAGS6502::I, true);
    self.bus.write_u8(
      0x100 + self.stack_pointer as u16,
      self.status_register,
    );
    self.stack_pointer -= 1;

    let address_abs = 0xFFFA;
    let lo_byte = self.bus.read_u8(address_abs);
    let hi_byte = self.bus.read_u8(address_abs + 1);
    self.pc = ((hi_byte << 8) | lo_byte) as u16;

    self.cycles = 8;
  }

  fn fetch_data() -> u8 {
    unimplemented!()
  }

  fn read_u16(&mut self, address: u16) -> u16 {
    let low_byte = self.bus.read_u8(address);
    let high_byte = self.bus.read_u8(address.wrapping_add(1));

    u16::from(low_byte) | u16::from(high_byte) << 8
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
    self.pc += 1;
    self.addr_abs = self.pc;
    0
  }

  pub fn zp0(&mut self) -> u8 {
    self.addr_abs = self.bus.read_u8(self.pc) as u16;
    self.pc += 1;
    self.addr_abs &= 0x00FF;
    0
  }

  pub fn zpx(&mut self) -> u8 {
    self.addr_abs = (self.bus.read_u8(self.pc) + self.x) as u16;
    self.pc += 1;
    self.addr_abs &= 0x00FF;
    0
  }

  pub fn zpy(&mut self) -> u8 {
    self.addr_abs = (self.bus.read_u8(self.pc) + self.y) as u16;
    self.pc += 1;
    self.addr_abs &= 0x00FF;
    0
  }

  pub fn rel(&mut self) -> u8 {
    self.addr_rel = self.bus.read_u8(self.pc) as u16;
    self.pc += 1;
    if self.addr_rel & 0x80 != 0 {
      self.addr_rel |= 0xFF00;
    }
    0
  }

  pub fn abs(&mut self) -> u8 {
    let lo_byte = self.bus.read_u8(self.pc);
    self.pc += 1;
    let hi_byte = self.bus.read_u8(self.pc);
    self.pc += 1;
    self.addr_abs = ((hi_byte << 8) | lo_byte) as u16;
    0
  }

  pub fn abx(&mut self) -> u8 {
    let lo_byte = self.bus.read_u8(self.pc);
    self.pc += 1;
    let hi_byte = self.bus.read_u8(self.pc);
    self.pc += 1;

    self.addr_abs = ((hi_byte << 8) | lo_byte) as u16;
    self.addr_abs += self.x as u16;
    if (self.addr_abs & 0xFF00) != (hi_byte << 8) as u16 {
      1
    } else {
      0
    }
  }

  pub fn aby(&mut self) -> u8 {
    let lo_byte = self.bus.read_u8(self.pc);
    self.pc += 1;
    let hi_byte = self.bus.read_u8(self.pc);
    self.pc += 1;

    self.addr_abs = ((hi_byte << 8) | lo_byte) as u16;
    self.addr_abs += self.y as u16;
    if (self.addr_abs & 0xFF00) != (hi_byte << 8) as u16 {
      1
    } else {
      0
    }
  }

  pub fn ind(&mut self) -> u8 {
    let lo_byte = self.bus.read_u8(self.pc);
    self.pc += 1;
    let hi_byte = self.bus.read_u8(self.pc);
    self.pc += 1;

    let byte = ((hi_byte << 8) | lo_byte) as u16;

    if lo_byte == 0x00FF {
      self.addr_abs = ((self.bus.read_u8(byte & 0xFF00) << 8) | self.bus.read_u8(byte + 0)) as u16;
    } else {
      self.addr_abs = (self.bus.read_u8((byte + 1) << 8) | self.bus.read_u8(byte + 0)) as u16
    }

    0
  }

  pub fn izx(&mut self) -> u8 {
    let byte = self.bus.read_u8(self.pc);
    self.pc += 1;

    let lo_byte = self.bus.read_u8((byte + self.x) as u16 & 0x00FF);
    let hi_byte = self.bus.read_u8((byte + self.x + 1) as u16 & 0x00FF);
    self.addr_abs = ((hi_byte << 8) | lo_byte) as u16;

    0
  }

  pub fn izy(&mut self) -> u8 {
    let byte = self.bus.read_u8(self.pc);
    self.pc += 1;

    let lo_byte = self.bus.read_u8((byte + self.x) as u16 & 0x00FF);
    let hi_byte = self.bus.read_u8((byte + self.x + 1) as u16 & 0x00FF);
    self.addr_abs = ((hi_byte << 8) | lo_byte) as u16;
    self.addr_abs += self.y as u16;


    if (self.addr_abs & 0xFF00) != (hi_byte << 8) as u16 {
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

  pub fn adc(&mut self) -> u8 {
    self.fetch();
    self.temp = self.a as u16 + self.fetched as u16 + self.get_flag(&FLAGS6502::C) as u16;

    self.set_flag(&FLAGS6502::C, self.temp > 255);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) != 0x00);
    self.set_flag(&FLAGS6502::V, (!(self.a ^ self.fetched) & ((self.a as u16) ^ self.temp) as u8) & 0x0080 != 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x80 != 0x00);
    self.a = (self.temp & 0x00FF) as u8;
    1
  }

  pub fn asl(&mut self) -> u8 {
    self.fetch();
    self.temp = (self.fetched as u16) << 1;
    self.set_flag(&FLAGS6502::C, (self.temp & 0xFF00) != 0x00);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0xFF00) == 0x00);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x80) != 0x00);

    let idx = usize::try_from(self.opcode).unwrap_or(0);
    let addr_mode = self.lookup.get_addr_mode(idx).clone();

    if addr_mode == ADDRMODE6502::IMP {
      self.a = (self.temp & 0x00FF) as u8;
    } else {
      self.bus.write_u8(self.addr_abs, (self.temp & 0x00FF) as u8);
    }
    0
  }
  pub fn and(&mut self) -> u8 {
    self.fetch();
    self.a = self.a & self.fetched;
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x80 != 0x00);
    1
  }
  pub fn bcc(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::C) == 0x00 {
      self.cycles += 1;
      self.addr_abs = self.pc + self.addr_rel;

      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles += 1;
      }
      self.pc = self.addr_abs;
    }
    0
  }

  pub fn bcs(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::C) > 0 {
      self.cycles += 1;
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles += 1;
      }
      self.pc = self.addr_abs;
    }
    0
  }

  pub fn beq(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::Z) > 0 {
      self.cycles += 1;
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles += 1;
      }
      self.pc = self.addr_abs;
    }
    0
  }

  pub fn bit(&mut self) -> u8 {
    self.fetch();
    self.temp = (self.a & self.fetched) as u16;
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) == 0x00);
    self.set_flag(&FLAGS6502::N, self.fetched & (1 << 7) != 0x00);
    self.set_flag(&FLAGS6502::V, self.fetched & (1 << 6) != 0x00);
    1
  }

  pub fn bmi(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::N) > 0 {
      self.cycles += 1;
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles += 1;
      }
      self.pc = self.addr_abs;
    }
    0
  }

  pub fn bne(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::Z) == 0 {
      self.cycles += 1;
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles += 1;
      }
      self.pc = self.addr_abs;
    }
    0
  }

  pub fn bpl(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::N) == 0 {
      self.cycles += 1;
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles += 1;
      }
      self.pc = self.addr_abs;
    }
    0
  }

  pub fn brk(&mut self) -> u8 {
    self.pc += 1;

    self.set_flag(&FLAGS6502::Z, true);
    self.bus.write_u8(0x0100 + self.stack_pointer as u16, (self.pc >> 8) as u8 & 0x00FF);
    self.stack_pointer -= 1;
    self.bus.write_u8(0x0100 + self.stack_pointer as u16, self.pc as u8 & 0x00FF);
    self.stack_pointer -= 1;

    self.set_flag(&FLAGS6502::B, true);
    self.bus.write_u8(0x0100 + self.stack_pointer as u16, self.pc as u8 & 0x00FF);
    self.stack_pointer -= 1;
    self.set_flag(&FLAGS6502::B, false);

    self.pc = self.bus.read_u8(0xFFFE) as u16 | (self.bus.read_u8(0xFFFF) as u16) << 8;
    0
  }

  pub fn bvc(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::V) == 0 {
      self.cycles += 1;
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles += 1;
      }
      self.pc = self.addr_abs;
    }
    0
  }

  pub fn bvs(&mut self) -> u8 {
    if self.get_flag(&FLAGS6502::V) == 1 {
      self.cycles += 1;
      self.addr_abs = self.pc + self.addr_rel;
      if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles += 1;
      }
      self.pc = self.addr_abs;
    }
    0
  }

  pub fn clc(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::C, false);
    0
  }

  pub fn cld(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::D, false);
    0
  }

  pub fn cli(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::I, false);
    0
  }

  pub fn clv(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::V, false);
    0
  }

  pub fn cmp(&mut self) -> u8 {
    self.fetch();
    self.temp = self.a as u16 - self.fetched as u16;
    self.set_flag(&FLAGS6502::C, self.a >= self.fetched);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 > 0x00);
    1
  }

  pub fn cpx(&mut self) -> u8 {
    self.fetch();
    self.temp = self.x as u16 - self.fetched as u16;
    self.set_flag(&FLAGS6502::C, self.x >= self.fetched);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 > 0x00);
    0
  }

  pub fn cpy(&mut self) -> u8 {
    self.fetch();
    self.temp = self.y as u16 - self.fetched as u16;
    self.set_flag(&FLAGS6502::C, self.y >= self.fetched);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 > 0x00);
    0
  }

  pub fn dec(&mut self) -> u8 {
    self.fetch();
    self.temp = self.fetched as u16 - 1;
    self.bus.write_u8(self.addr_abs, self.temp as u8 & 0x00FF);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 > 0x00);
    0
  }

  pub fn dex(&mut self) -> u8 {
    self.x -= 1;
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, self.x == 0x80);
    0
  }

  pub fn dey(&mut self) -> u8 {
    self.y -= 1;
    self.set_flag(&FLAGS6502::Z, self.y == 0x00);
    self.set_flag(&FLAGS6502::N, self.y == 0x80);
    0
  }

  pub fn eor(&mut self) -> u8 {
    self.fetch();
    self.a = self.a ^ self.fetched;
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x80 > 0x00);
    1
  }

  pub fn inc(&mut self) -> u8 {
    self.fetch();
    self.temp = self.fetched as u16 + 1;
    self.bus.write_u8(self.addr_abs, self.temp as u8 & 0x00FF);
    self.set_flag(&FLAGS6502::Z, (self.temp & 0x00FF) == 0x00);
    self.set_flag(&FLAGS6502::N, (self.temp & 0x0080) > 0x00);
    0
  }

  pub fn inx(&mut self) -> u8 {
    self.x += 1;
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, self.x & 0x80 > 0x00);
    0
  }

  pub fn iny(&mut self) -> u8 {
    self.y += 1;
    self.set_flag(&FLAGS6502::Z, self.y == 0x00);
    self.set_flag(&FLAGS6502::N, self.y & 0x80 > 0x00);
    0
  }

  pub fn jmp(&mut self) -> u8 {
    self.pc = self.addr_abs;
    0
  }

  pub fn jsr(&mut self) -> u8 {
    self.pc -= 1;
    self.bus.write_u8(0x0100 & self.stack_pointer as u16, (self.pc >> 8) as u8 & 0x00FF);
    self.stack_pointer -= 1;
    self.bus.write_u8(0x0100 & self.stack_pointer as u16, self.pc as u8 & 0x00FF);
    self.stack_pointer -= 1;

    self.pc = self.addr_abs;
    0
  }

  pub fn lda(&mut self) -> u8 {
    self.fetch();
    self.a = self.fetched;
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x80 > 0x00);
    1
  }

  pub fn ldx(&mut self) -> u8 {
    self.fetch();
    self.x = self.fetched;
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, self.x & 0x80 > 0x00);
    1
  }

  pub fn ldy(&mut self) -> u8 {
    self.fetch();
    self.y = self.fetched;
    self.set_flag(&FLAGS6502::Z, self.y == 0x00);
    self.set_flag(&FLAGS6502::N, self.y & 0x80 > 0x00);
    1
  }

  pub fn lsr(&mut self) -> u8 {
    self.fetch();
    self.set_flag(&FLAGS6502::C, self.fetched & 0x0001 > 0x00);
    self.temp = self.fetched as u16 >> 1;
    self.set_flag(&FLAGS6502::Z, self.temp & 0x00FF == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 > 0x00);

    let idx = usize::try_from(self.opcode).unwrap_or(0);
    let addr_mode = self.lookup.get_addr_mode(idx).clone();

    if addr_mode == ADDRMODE6502::IMP {
      self.a = self.temp as u8 & 0x00FF;
    } else {
      self.bus.write_u8(self.addr_abs, self.temp as u8 & 0x00FF);
    }
    0
  }

  pub fn nop(&mut self) -> u8 {
    match self.opcode {
      0x1C => 1,
      0x3C => 1,
      0x5C => 1,
      0x7C => 1,
      0xDC => 1,
      0xFC => 1,
      _ => 0
    }
  }

  pub fn ora(&mut self) -> u8 {
    self.fetch();
    self.a = self.a | self.fetched;
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x0080 > 0x00);
    1
  }

  pub fn pha(&mut self) -> u8 {
    self.bus.write_u8(0x0100 + self.stack_pointer as u16, self.a);
    self.stack_pointer -= 1;
    0
  }

  pub fn php(&mut self) -> u8 {
    self.bus.write_u8(0x0100 + self.stack_pointer as u16, self.status_register | FLAGS6502::B.value() | FLAGS6502::U.value());
    self.set_flag(&FLAGS6502::B, false);
    self.set_flag(&FLAGS6502::U, false);
    self.stack_pointer -= 1;
    0
  }

  pub fn pla(&mut self) -> u8 {
    self.stack_pointer += 1;
    self.a = self.bus.read_u8(0x0100 + self.stack_pointer as u16);
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x80 > 0x00);
    0
  }

  pub fn plp(&mut self) -> u8 {
    self.stack_pointer += 1;
    self.status_register = self.bus.read_u8(0x0100 + self.stack_pointer as u16);
    self.set_flag(&FLAGS6502::U, true);
    0
  }

  pub fn rol(&mut self) -> u8 {
    self.fetch();
    self.temp = (self.fetched << 1) as u16 | self.get_flag(&FLAGS6502::C) as u16;
    self.set_flag(&FLAGS6502::C, self.temp & 0xFF00 > 0x00);
    self.set_flag(&FLAGS6502::Z, self.temp & 0x00FF == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 > 0x00);

    let idx = usize::try_from(self.opcode).unwrap_or(0);
    let addr_mode = self.lookup.get_addr_mode(idx).clone();
    if addr_mode == ADDRMODE6502::IMP {
      self.a = self.temp as u8 & 0x00FF;
    } else {
      self.bus.write_u8(self.addr_abs, self.temp as u8 & 0x00FF);
    }
    0
  }

  pub fn ror(&mut self) -> u8 {
    self.fetch();
    self.temp = self.get_flag(&FLAGS6502::C) as u16 | (self.fetched << 1) as u16;
    self.set_flag(&FLAGS6502::C, self.fetched & 0x01 > 0x00);
    self.set_flag(&FLAGS6502::Z, self.temp & 0x00FF == 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 > 0x00);

    let idx = usize::try_from(self.opcode).unwrap_or(0);
    let addr_mode = self.lookup.get_addr_mode(idx).clone();
    if addr_mode == ADDRMODE6502::IMP {
      self.a = self.temp as u8 & 0x00FF;
    } else {
      self.bus.write_u8(self.addr_abs, self.temp as u8 & 0x00FF);
    }
    0
  }

  pub fn rti(&mut self) -> u8 {
    self.stack_pointer += 1;
    self.status_register = self.bus.read_u8(0x0100 + self.stack_pointer as u16);
    self.status_register &= !FLAGS6502::B.value();
    self.status_register &= !FLAGS6502::U.value();

    self.stack_pointer += 1;
    self.pc = self.bus.read_u8(0x0100 + self.stack_pointer as u16) as u16;
    self.stack_pointer += 1;
    self.pc |= (self.bus.read_u8(0x0100 + self.stack_pointer as u16) as u16) << 8;
    0
  }

  pub fn rts(&mut self) -> u8 {
    self.stack_pointer += 1;
    self.pc = self.bus.read_u8(0x0100 + self.stack_pointer as u16) as u16;
    self.stack_pointer += 1;
    self.pc |= (self.bus.read_u8(0x0100 + self.stack_pointer as u16) as u16) << 8;

    self.pc += 1;
    0
  }

  pub fn sbc(&mut self) -> u8 {
    self.fetch();
    let value = self.fetched ^ 0x00FF;

    self.temp = (self.a + value + self.get_flag(&FLAGS6502::C)) as u16;
    self.set_flag(&FLAGS6502::C, self.temp & 0xFF00 != 0x00);
    self.set_flag(&FLAGS6502::Z, self.temp & 0x00FF == 0);
    self.set_flag(&FLAGS6502::C, (self.temp ^ self.a as u16 & (self.temp ^ value as u16) & 0x0080) != 0x00);
    self.set_flag(&FLAGS6502::N, self.temp & 0x0080 != 0x00);
    self.a = self.temp as u8 & 0x00FF;
    1
  }

  pub fn sec(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::D, true);
    0
  }

  pub fn sed(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::D, true);
    0
  }

  pub fn sei(&mut self) -> u8 {
    self.set_flag(&FLAGS6502::I, true);
    0
  }

  pub fn sta(&mut self) -> u8 {
    self.bus.write_u8(self.addr_abs, self.y);
    0
  }

  pub fn stx(&mut self) -> u8 {
    self.bus.write_u8(self.addr_abs, self.x);
    0
  }

  pub fn sty(&mut self) -> u8 {
    self.bus.write_u8(self.addr_abs, self.y);
    0
  }

  pub fn tax(&mut self) -> u8 {
    self.x = self.a;
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, self.x & 0x80 > 0x00);
    0
  }

  pub fn tay(&mut self) -> u8 {
    self.y = self.a;
    self.set_flag(&FLAGS6502::Z, self.y == 0x00);
    self.set_flag(&FLAGS6502::N, self.y & 0x80 > 0x00);
    0
  }

  pub fn tsx(&mut self) -> u8 {
    self.x = self.stack_pointer;
    self.set_flag(&FLAGS6502::Z, self.x == 0x00);
    self.set_flag(&FLAGS6502::N, self.x & 0x80 > 0x00);
    0
  }

  pub fn txa(&mut self) -> u8 {
    self.a = self.x;
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x80 > 0x00);
    0
  }

  pub fn txs(&mut self) -> u8 {
    self.stack_pointer = self.x;
    0
  }

  pub fn tya(&mut self) -> u8 {
    self.a = self.y;
    self.set_flag(&FLAGS6502::Z, self.a == 0x00);
    self.set_flag(&FLAGS6502::N, self.a & 0x80 > 0x00);
    0
  }

  pub fn disassemble(&self, start: u16, end: u16) -> HashMap<u16, String> {
    let mut addr = start;
    let value: u8 = 0x00;
    let mut lo_byte: u8 = 0x00;
    let mut hi_byte: u8 = 0x00;

    let mut map: HashMap<u16, String> = HashMap::new();
    let mut line_addr: u16 = 0x00;

    fn hex(n: usize, d: usize) -> String {
      let mut output = Vec::with_capacity(d);
      for x in (0..d - 1).step_by(4) {
        output[x] = "0123456789ABCDEF".chars().nth(n & 0xF).unwrap_or(' ');
      }
      output.into_iter().map(|c| c.to_string()).collect()
    }

    while addr <= end {
      line_addr = addr;

      let mut codes = format!("$ {}:", hex(addr as usize, 4));
      let opcode = self.bus.read_u8(addr);
      addr += 1;

      let idx = usize::try_from(opcode).unwrap_or(0);
      let name = self.lookup.get_name(idx);

      codes = format!("{} {} ", codes, addr);

      let addr_mode = self.lookup.get_addr_mode(idx).clone();

      match addr_mode {
        ADDRMODE6502::IMP => {
          codes = format!("{} {{IMP}} ", codes);
        },
        ADDRMODE6502::IMM => {
          let value = self.bus.read_u8(addr);
          addr += 1;
          codes = format!("{}$ {} {{IMM}} ", codes, hex(value as usize, 2));
        },
        ADDRMODE6502::ZP0 => {
          lo_byte = self.bus.read_u8(addr);
          addr += 1;
          hi_byte = 0x00;
          codes = format!("{}$ {} {{ZP0}} ", codes, hex(value as usize, 2));
        },
        ADDRMODE6502::ZPX => {
          lo_byte = self.bus.read_u8(addr);
          addr += 1;
          hi_byte = 0x00;
          codes = format!("{}$ {} {{ZPX}} ", codes, hex(value as usize, 2));
        },
        ADDRMODE6502::ZPY => {
          lo_byte = self.bus.read_u8(addr);
          addr += 1;
          hi_byte = 0x00;
          codes = format!("{}$ {} {{ZPY}} ", codes, hex(value as usize, 2));
        },
        ADDRMODE6502::REL => {
          let value = self.bus.read_u8(addr);
          addr += 1;
          codes = format!("{}$ {} {{ZP0}} ", codes, hex(value as usize, 2));
        },
        ADDRMODE6502::ABS => {
          lo_byte = self.bus.read_u8(addr);
          addr += 1;
          hi_byte = self.bus.read_u8(addr);
          addr += 1;
          codes = format!("{}$ {} {{ABS}} ", codes, hex(usize::from(hi_byte << 8 | lo_byte), 4));
        },
        ADDRMODE6502::ABX => {
          lo_byte = self.bus.read_u8(addr);
          addr += 1;
          hi_byte = self.bus.read_u8(addr);
          addr += 1;
          codes = format!("{}$ {} {{ABS}} ", codes, hex(usize::from(hi_byte << 8 | lo_byte), 4));
        },
        ADDRMODE6502::ABY => {
          lo_byte = self.bus.read_u8(addr);
          addr += 1;
          hi_byte = self.bus.read_u8(addr);
          addr += 1;
          codes = format!("{}$ {} {{ABY}} ", codes, hex(usize::from(hi_byte << 8 | lo_byte), 4));
        },
        ADDRMODE6502::IND => {
          lo_byte = self.bus.read_u8(addr);
          addr += 1;
          hi_byte = self.bus.read_u8(addr);
          addr += 1;
          codes = format!("{}$ {} {{IND}} ", codes, hex(usize::from(hi_byte << 8 | lo_byte), 4));
        },
        ADDRMODE6502::IZX => {
          lo_byte = self.bus.read_u8(addr);
          addr += 1;
          hi_byte = 0x00;
          codes = format!("{}$ {} {{IZX}} ", codes, hex(value as usize, 2));
        },
        ADDRMODE6502::IZY => {
          lo_byte = self.bus.read_u8(addr);
          addr += 1;
          hi_byte = 0x00;
          codes = format!("{}$ {} {{IZY}} ", codes, hex(value as usize, 2));
        },
      }

      map.insert(line_addr, codes);
    }
    map
  }
}

use std::convert::{TryFrom, TryInto};

use crate::bus::Bus;
use crate::cpu::instruction_table::{AddrMode6502, Flag6502, hex, LookUpTable, OpCode6502};

pub mod instruction_table;
#[cfg(test)]
mod cpu_test;

pub struct Cpu {
  pub bus: Bus,
  pub pc: u16,
  pub acc: u8,
  pub x: u8,
  pub y: u8,
  pub status_register: u8,
  pub stack_pointer: u8,
  fetched: u8,
  addr_abs: u16,
  addr_rel: u16,
  pub opcode: u8,
  pub cycle: u8,
  lookup: LookUpTable,
  system_cycle: u32,
}

impl Cpu {
  pub fn new(bus: Bus) -> Cpu {
    let lookup = LookUpTable::new();

    Cpu {
      bus,
      pc: 0,
      acc: 0,
      x: 0,
      y: 0,
      fetched: 0x0,
      addr_abs: 0x0u16,
      addr_rel: 0x0u16,
      opcode: 0x0u8,
      stack_pointer: 0x0u8,
      status_register: 0u8,
      cycle: 0u8,
      lookup,
      system_cycle: 0,
    }
  }

  fn set_flag(&mut self, flag: &Flag6502, val: bool) {
    let f = flag.value();
    if val {
      self.status_register |= f;
    } else {
      self.status_register &= !f;
    }
  }

  fn get_flag(&self, flag: &Flag6502) -> bool {
    (self.status_register & flag.value()) > 0
  }

  fn get_flag_val(&self, flag: &Flag6502) -> u16 {
    //if (self.status_register & flag.value()) > 0 { 1 } else { 0 }
    u16::from(self.status_register & flag.value() > 0)
  }

  pub fn bus_mut_read_u8(&mut self, address: u16) -> u8 {
    self.bus.read_u8(address) as u8
  }

  pub fn bus_mut_read_dbg_u8(&mut self, address_start: usize, address_end: usize) -> Vec<u8> {
    self.bus.read_dbg_u8(address_start, address_end)
  }

  fn bus_write_u8(&mut self, address: u16, data: u8) {
    self.bus.write_u8(address, data, self.system_cycle);
  }

  fn get_stack_address(&self) -> u16 {
    0x0100 + u16::try_from(self.stack_pointer).unwrap()
  }

  fn pc_increment(&mut self) {
    self.pc = self.pc.wrapping_add(1);
  }

  fn cycles_increment(&mut self) {
    self.cycle = self.cycle.wrapping_add(1);
  }

  fn stack_pointer_increment(&mut self) {
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
  }

  fn stack_pointer_decrement(&mut self) {
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
  }

  fn set_flags_zero_and_negative(&mut self, val: u16) {
    self.set_flag(&Flag6502::Z, (val & 0x00FF) == 0x00);
    self.set_flag(&Flag6502::N, (val & 0x0080) > 0);
  }

  fn branching(&mut self, condition: bool) -> u8 {
    if condition {
      self.cycles_increment();
      let new_pc = self.pc.wrapping_add(self.addr_rel);
      if (new_pc & 0xFF00) != (self.pc & 0xFF00) {
        self.cycles_increment();
      }
      self.pc = new_pc;
    }
    0
  }

  fn return_or_write_memory(&mut self, val: u16) {
    if self.addr_mode() == AddrMode6502::Imp {
      self.acc = u8::try_from(val & 0xFF).unwrap();
    } else {
      self.bus_write_u8(self.addr_abs, u8::try_from(val & 0xFF).unwrap());
    }
  }

  pub fn clock(&mut self, system_cycle: u32) {
      self.system_cycle = system_cycle;
      self.opcode = self.bus_mut_read_u8(self.pc);

      self.pc_increment();

      let opcode_idx = usize::try_from(self.opcode).unwrap();
      self.cycle = self.lookup.get_cycles(opcode_idx);

      let addr_mode = *self.lookup.get_addr_mode(opcode_idx);
      let operate = *self.lookup.get_operate(opcode_idx);

      self.cycle += self.addr_mode_value(addr_mode) & self.op_code_value(operate);
  }

  #[allow(dead_code)]
  fn log(&self, log_pc: usize) {
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut file = OpenOptions::new()
      .write(true)
      .append(true)
      .open("log.txt")
      .expect("File append error");

    file
      .write_all(
        format!(
          "opcode:{} {},{} PC:{} XXX A:{} X:{} Y:{} {}{}{}{}{}{}{}{} STKP:{}\n",
          self.opcode,
          self.addr_abs,
          self.addr_rel,
          hex(log_pc, 4),
          hex(u8::try_into(self.acc).unwrap(), 2),
          hex(u8::try_into(self.x).unwrap(), 2),
          hex(u8::try_into(self.y).unwrap(), 2),
          if self.get_flag(&Flag6502::N) { "N" } else { "." },
          if self.get_flag(&Flag6502::V) { "V" } else { "." },
          if self.get_flag(&Flag6502::U) { "U" } else { "." },
          if self.get_flag(&Flag6502::B) { "B" } else { "." },
          if self.get_flag(&Flag6502::D) { "D" } else { "." },
          if self.get_flag(&Flag6502::I) { "I" } else { "." },
          if self.get_flag(&Flag6502::Z) { "Z" } else { "." },
          if self.get_flag(&Flag6502::C) { "C" } else { "." },
          hex(usize::try_from(self.stack_pointer).unwrap(), 2),
        )
          .as_bytes(),
      )
      .expect("File write error");
  }

  pub fn fetch(&mut self) {
    if self.addr_mode() != AddrMode6502::Imp {
      self.fetched = self.bus_mut_read_u8(self.addr_abs);
    }
  }

  pub fn reset(&mut self) {
    self.addr_abs = 0xFFFC;

    let lo_byte = self.bus_mut_read_u8(self.addr_abs) as u16;
    let hi_byte = self.bus_mut_read_u8(self.addr_abs.wrapping_add(1)) as u16;

    self.pc = (hi_byte << 8) | lo_byte;
    self.acc = 0;
    self.x = 0;
    self.y = 0;
    self.stack_pointer = 0xFD;
    self.status_register = Flag6502::U.value();

    self.addr_abs = 0x0000;
    self.addr_rel = 0x0000;
    self.fetched = 0x00;

    self.cycle = 8;
  }

  pub fn irq(&mut self) {
    if self.get_flag(&Flag6502::I) || self.bus.get_mut_apu().get_irq_flag() {
      self.bus_write_u8(self.get_stack_address(), u8::try_from((self.pc >> 8) & 0x00FF).unwrap());
      self.stack_pointer_decrement();
      self.bus_write_u8(self.get_stack_address(), u8::try_from(self.pc & 0x00FF).unwrap());
      self.stack_pointer_decrement();

      self.set_flag(&Flag6502::B, false);
      self.set_flag(&Flag6502::U, true);
      self.set_flag(&Flag6502::I, true);
      self.bus_write_u8(self.get_stack_address(), self.status_register);
      self.stack_pointer_decrement();

      self.addr_abs = 0xFFFE;
      let lo_byte = self.bus_mut_read_u8(self.addr_abs) as u16;
      let hi_byte = self.bus_mut_read_u8(self.addr_abs + 1) as u16;
      self.pc = (hi_byte << 8) | lo_byte;

      self.cycle = 7;
    }
  }

  /// Non-maskable interrupt
  pub fn nmi(&mut self) {
    self.bus_write_u8(self.get_stack_address(), u8::try_from((self.pc >> 8) & 0xFF).unwrap());
    self.stack_pointer_decrement();
    self.bus_write_u8(self.get_stack_address(), u8::try_from(self.pc & 0xFF).unwrap());
    self.stack_pointer_decrement();

    self.set_flag(&Flag6502::B, false);
    self.set_flag(&Flag6502::U, true);
    self.set_flag(&Flag6502::I, true);

    self.bus_write_u8(self.get_stack_address(), self.status_register);
    self.stack_pointer_decrement();

    self.addr_abs = 0xFFFA;
    let lo_byte = self.bus_mut_read_u8(self.addr_abs) as u16;
    let hi_byte = self.bus_mut_read_u8(self.addr_abs.wrapping_add(1)) as u16;
    self.pc = (hi_byte << 8) | lo_byte;

    self.cycle = 8;
  }

  /// ADDRESS MODES
  pub fn addr_mode_value(&mut self, addr_mode: AddrMode6502) -> u8 {
    match addr_mode {
      AddrMode6502::Imp => self.imp(),
      AddrMode6502::Imm => self.imm(),
      AddrMode6502::Zpo => self.zp0(),
      AddrMode6502::Zpx => self.zpx(),
      AddrMode6502::Zpy => self.zpy(),
      AddrMode6502::Rel => self.rel(),
      AddrMode6502::Abs => self.abs(),
      AddrMode6502::Abx => self.abx(),
      AddrMode6502::Aby => self.aby(),
      AddrMode6502::Ind => self.ind(),
      AddrMode6502::Izx => self.izx(),
      AddrMode6502::Izy => self.izy(),
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
    self.addr_abs = self.bus_mut_read_u8(self.pc) as u16;
    self.pc_increment();
    self.addr_abs &= 0x00FF;
    0
  }

  /// Zero Page with X offset
  pub fn zpx(&mut self) -> u8 {
    self.addr_abs = self.bus_mut_read_u8(self.pc).wrapping_add(self.x) as u16;
    self.pc_increment();
    self.addr_abs &= 0x00FF;
    0
  }

  /// Zero Page with Y offset
  pub fn zpy(&mut self) -> u8 {
    self.addr_abs = self.bus_mut_read_u8(self.pc).wrapping_add(self.y) as u16;
    self.pc_increment();
    self.addr_abs &= 0x00FF;
    0
  }

  /// Relative
  pub fn rel(&mut self) -> u8 {
    self.addr_rel = self.bus_mut_read_u8(self.pc) as u16;
    self.pc_increment();
    if (self.addr_rel & 0x80) > 0 {
      self.addr_rel |= 0xFF00;
    }
    0
  }

  /// Absolute
  pub fn abs(&mut self) -> u8 {
    let (lo_byte, hi_byte) = self.read_pc();
    self.addr_abs = (hi_byte | lo_byte) as u16;
    0
  }

  /// Absolute with X offset
  pub fn abx(&mut self) -> u8 {
    let (lo_byte, hi_byte) = self.read_pc();

    self.addr_abs = (hi_byte | lo_byte) as u16;
    self.addr_abs = self.addr_abs.wrapping_add(u16::try_from(self.x).unwrap());
    u8::from((self.addr_abs & 0xFF00) != hi_byte)
  }

  /// Absolute with Y offset
  pub fn aby(&mut self) -> u8 {
    let (lo_byte, hi_byte) = self.read_pc();

    self.addr_abs = (hi_byte | lo_byte) as u16;
    self.addr_abs = self.addr_abs.wrapping_add(u16::try_from(self.y).unwrap());
    u8::from((self.addr_abs & 0xFF00) != hi_byte)
  }

  /// Indirect
  pub fn ind(&mut self) -> u8 {
    let (lo_byte, hi_byte) = self.read_pc();

    let byte = hi_byte | lo_byte;
    let b = if lo_byte == 0x00FF { byte & 0xFF00 } else { byte.wrapping_add(1) };
    self.addr_abs = (u16::try_from(self.bus_mut_read_u8(b)).unwrap() << 8) | u16::try_from(self.bus_mut_read_u8(byte)).unwrap();

    0
  }

  /// Indirect X
  pub fn izx(&mut self) -> u8 {
    let byte = self.bus_mut_read_u8(self.pc) as u16;
    self.pc_increment();

    let x = u16::try_from(self.x).unwrap();
    let lo_byte = self.bus_mut_read_u8(byte.wrapping_add(x) & 0x00FF) as u16;
    let hi_byte = self.bus_mut_read_u8((byte.wrapping_add(x).wrapping_add(1)) & 0x00FF) as u16;
    self.addr_abs = ((hi_byte << 8) | lo_byte) as u16;

    0
  }

  /// Indirect Y
  pub fn izy(&mut self) -> u8 {
    let byte = self.bus_mut_read_u8(self.pc) as u16;
    self.pc_increment();

    let lo_byte = self.bus_mut_read_u8(byte & 0xFF) as u16;
    let hi_byte = self.bus_mut_read_u8((byte.wrapping_add(1)) & 0xFF) as u16;
    self.addr_abs = (hi_byte << 8) | lo_byte;
    self.addr_abs = self.addr_abs.wrapping_add(u16::try_from(self.y).unwrap());

    ((self.addr_abs & 0xFF00) != (hi_byte << 8)).into()
  }

  fn read_pc(&mut self) -> (u16, u16) {
    let lo_byte = self.bus_mut_read_u8(self.pc) as u16;
    self.pc_increment();
    let hi_byte = self.bus_mut_read_u8(self.pc) as u16;
    self.pc_increment();
    (lo_byte, (hi_byte << 8))
  }

  /// OP CODES
  pub fn op_code_value(&mut self, op_code: OpCode6502) -> u8 {
    match op_code {
      OpCode6502::Add => self.adc(),
      OpCode6502::And => self.and(),
      OpCode6502::Asl => self.asl(),
      OpCode6502::Bcc => self.bcc(),
      OpCode6502::Bcs => self.bcs(),
      OpCode6502::Beq => self.beq(),
      OpCode6502::Bit => self.bit(),
      OpCode6502::Bmi => self.bmi(),
      OpCode6502::Bne => self.bne(),
      OpCode6502::Bpl => self.bpl(),
      OpCode6502::Brk => self.brk(),
      OpCode6502::Bvc => self.bvc(),
      OpCode6502::Bvs => self.bvs(),
      OpCode6502::Clc => self.clc(),
      OpCode6502::Cld => self.cld(),
      OpCode6502::Cli => self.cli(),
      OpCode6502::Clv => self.clv(),
      OpCode6502::Cmp => self.cmp(),
      OpCode6502::Cpx => self.cpx(),
      OpCode6502::Cpy => self.cpy(),
      OpCode6502::Dec => self.dec(),
      OpCode6502::Dex => self.dex(),
      OpCode6502::Dey => self.dey(),
      OpCode6502::Eor => self.eor(),
      OpCode6502::Inc => self.inc(),
      OpCode6502::Inx => self.inx(),
      OpCode6502::Iny => self.iny(),
      OpCode6502::Jmp => self.jmp(),
      OpCode6502::Jsr => self.jsr(),
      OpCode6502::Lda => self.lda(),
      OpCode6502::Ldx => self.ldx(),
      OpCode6502::Ldy => self.ldy(),
      OpCode6502::Lsr => self.lsr(),
      OpCode6502::Nop => self.nop(),
      OpCode6502::Ora => self.ora(),
      OpCode6502::Pha => self.pha(),
      OpCode6502::Php => self.php(),
      OpCode6502::Pla => self.pla(),
      OpCode6502::Plp => self.plp(),
      OpCode6502::Rol => self.rol(),
      OpCode6502::Ror => self.ror(),
      OpCode6502::Rti => self.rti(),
      OpCode6502::Rts => self.rts(),
      OpCode6502::Sbc => self.sbc(),
      OpCode6502::Sec => self.sec(),
      OpCode6502::Sed => self.sed(),
      OpCode6502::Sei => self.sei(),
      OpCode6502::Sta => self.sta(),
      OpCode6502::Stx => self.stx(),
      OpCode6502::Sty => self.sty(),
      OpCode6502::Tax => self.tax(),
      OpCode6502::Tay => self.tay(),
      OpCode6502::Tsx => self.tsx(),
      OpCode6502::Txa => self.txa(),
      OpCode6502::Txs => self.txs(),
      OpCode6502::Tya => self.tya(),
      OpCode6502::Xxx => 0,
    }
  }

  /// Add with carry
  pub fn adc(&mut self) -> u8 {
    self.fetch();
    let val = u16::try_from(self.acc).unwrap()
      .wrapping_add(u16::try_from(self.fetched).unwrap())
      .wrapping_add(self.get_flag_val(&Flag6502::C));

    self.set_flag(&Flag6502::C, (val) > 255);
    self.set_flag(
      &Flag6502::V,
      ((!(u16::try_from(self.acc).unwrap()
        ^ u16::try_from(self.fetched).unwrap())
        & (u16::try_from(self.acc).unwrap() ^ val))
        & 0x80)
        > 0,
    );
    self.set_flags_zero_and_negative(val & 0xFF);
    self.acc = u8::try_from(val & 0xFF).unwrap();
    1
  }

  /// Arithmetic shift left
  pub fn asl(&mut self) -> u8 {
    self.fetch();
    let val = u16::try_from(self.fetched).unwrap() << 1;
    self.set_flag(&Flag6502::C, (val & 0xFF00) > 0);
    self.set_flag(&Flag6502::Z, val.trailing_zeros() > 7);
    self.set_flag(&Flag6502::N, (val & 0x80) > 0);

    self.return_or_write_memory(val);
    0
  }

  /// and (with accumulator)
  pub fn and(&mut self) -> u8 {
    self.fetch();
    self.acc &= self.fetched;
    self.set_flags_zero_and_negative(self.acc.into());
    1
  }

  /// branch on carry clear
  pub fn bcc(&mut self) -> u8 {
    self.branching(!self.get_flag(&Flag6502::C))
  }

  /// branch on carry clear
  pub fn bcs(&mut self) -> u8 {
    self.branching(self.get_flag(&Flag6502::C))
  }

  /// branch on equal
  pub fn beq(&mut self) -> u8 {
    self.branching(self.get_flag(&Flag6502::Z))
  }

  /// Bit test
  pub fn bit(&mut self) -> u8 {
    self.fetch();
    let val = u16::try_from(self.acc).unwrap() & u16::try_from(self.fetched).unwrap();
    self.set_flag(&Flag6502::Z, val.trailing_zeros() > 7);
    self.set_flag(&Flag6502::N, (self.fetched & 0x80) > 0);
    self.set_flag(&Flag6502::V, (self.fetched & 0x40) > 0);
    0
  }

  /// Branch on minus (negative set)
  pub fn bmi(&mut self) -> u8 {
    self.branching(self.get_flag(&Flag6502::N))
  }

  /// Branch on not equal (zero clear)
  pub fn bne(&mut self) -> u8 {
    self.branching(!self.get_flag(&Flag6502::Z))
  }

  /// Branch on plus (negative clear)
  pub fn bpl(&mut self) -> u8 {
    self.branching(!self.get_flag(&Flag6502::N))
  }

  /// Break / interrupt
  pub fn brk(&mut self) -> u8 {
    self.pc_increment();

    self.set_flag(&Flag6502::I, true);
    self.bus_write_u8(self.get_stack_address(), u8::try_from((self.pc >> 8) & 0xFF).unwrap());
    self.stack_pointer_decrement();

    self.bus_write_u8(self.get_stack_address(), u8::try_from(self.pc & 0xFF).unwrap());
    self.stack_pointer_decrement();

    self.set_flag(&Flag6502::B, true);
    self.bus_write_u8(self.get_stack_address(), self.status_register);
    self.stack_pointer_decrement();
    self.set_flag(&Flag6502::B, false);

    self.pc = u16::try_from(self.bus_mut_read_u8(0xFFFE)).unwrap() | u16::try_from(self.bus_mut_read_u8(0xFFFF)).unwrap() << 8;
    0
  }

  /// Branch on overflow clear
  pub fn bvc(&mut self) -> u8 {
    self.branching(!self.get_flag(&Flag6502::V))
  }

  /// Branch on overflow clear
  pub fn bvs(&mut self) -> u8 {
    self.branching(self.get_flag(&Flag6502::V))
  }

  /// Clear carry
  pub fn clc(&mut self) -> u8 {
    self.set_flag(&Flag6502::C, false);
    0
  }

  /// Clear decimal
  pub fn cld(&mut self) -> u8 {
    self.set_flag(&Flag6502::D, false);
    0
  }

  /// Clear interrupt disable
  pub fn cli(&mut self) -> u8 {
    self.set_flag(&Flag6502::I, false);
    0
  }

  /// Clear overflow
  pub fn clv(&mut self) -> u8 {
    self.set_flag(&Flag6502::V, false);
    0
  }

  /// Compare with accumulator
  pub fn cmp(&mut self) -> u8 {
    self.fetch();
    let val = u16::try_from(self.acc.wrapping_sub(self.fetched)).unwrap();
    self.set_flag(&Flag6502::C, self.acc >= self.fetched);
    self.set_flags_zero_and_negative(val);
    1
  }

  /// Compare with X
  pub fn cpx(&mut self) -> u8 {
    self.fetch();
    let val = u16::try_from(self.x.wrapping_sub(self.fetched)).unwrap();
    self.set_flag(&Flag6502::C, self.x >= self.fetched);
    self.set_flags_zero_and_negative(val);
    0
  }

  /// Compare with Y
  pub fn cpy(&mut self) -> u8 {
    self.fetch();
    let val = u16::try_from(self.y.wrapping_sub(self.fetched)).unwrap();
    self.set_flag(&Flag6502::C, self.y >= self.fetched);
    self.set_flags_zero_and_negative(val);
    0
  }

  /// Decrement
  pub fn dec(&mut self) -> u8 {
    self.fetch();
    let val = u16::try_from(self.fetched).unwrap().wrapping_sub(1);
    self.bus_write_u8(self.addr_abs, u8::try_from(val & 0xFF).unwrap());
    self.set_flags_zero_and_negative(val);
    0
  }

  /// Decrement X
  pub fn dex(&mut self) -> u8 {
    self.x = self.x.wrapping_sub(1);
    self.set_flags_zero_and_negative(self.x.into());
    0
  }

  /// Decrement Y
  pub fn dey(&mut self) -> u8 {
    self.y = self.y.wrapping_sub(1);
    self.set_flags_zero_and_negative(self.y.into());
    0
  }

  /// Exclusive or with accumulator
  pub fn eor(&mut self) -> u8 {
    self.fetch();
    self.acc ^= self.fetched;
    self.set_flags_zero_and_negative(self.acc.into());
    1
  }

  /// Increment
  pub fn inc(&mut self) -> u8 {
    self.fetch();
    let val = u16::try_from(self.fetched.wrapping_add(1)).unwrap();
    self.bus_write_u8(self.addr_abs, u8::try_from(val & 0xFF).unwrap());
    self.set_flags_zero_and_negative(val);
    0
  }

  /// Increment X
  pub fn inx(&mut self) -> u8 {
    self.x = self.x.wrapping_add(1);
    self.set_flags_zero_and_negative(self.x.into());
    0
  }

  /// Increment Y
  pub fn iny(&mut self) -> u8 {
    self.y = self.y.wrapping_add(1);
    self.set_flags_zero_and_negative(self.y.into());
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
      u8::try_from((self.pc >> 8) & 0xFF).unwrap());
    self.stack_pointer_decrement();
    self.bus_write_u8(self.get_stack_address(),
                      u8::try_from(self.pc & 0xFF).unwrap());
    self.stack_pointer_decrement();

    self.pc = self.addr_abs;
    0
  }

  /// Load accumulator
  pub fn lda(&mut self) -> u8 {
    self.fetch();
    self.acc = self.fetched;
    self.set_flags_zero_and_negative(self.acc.into());
    1
  }

  /// Load X
  pub fn ldx(&mut self) -> u8 {
    self.fetch();
    self.x = self.fetched;
    self.set_flags_zero_and_negative(self.x.into());
    1
  }

  /// Load Y
  pub fn ldy(&mut self) -> u8 {
    self.fetch();
    self.y = self.fetched;
    self.set_flags_zero_and_negative(self.y.into());
    1
  }

  /// Logical shift right
  pub fn lsr(&mut self) -> u8 {
    self.fetch();
    self.set_flag(&Flag6502::C, (self.fetched & 1) > 0);
    let val = u16::try_from(self.fetched >> 1).unwrap();
    self.set_flags_zero_and_negative(val);

    self.return_or_write_memory(val);
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
    self.set_flags_zero_and_negative(self.acc.into());
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
      self.status_register | Flag6502::B.value() | Flag6502::U.value());
    self.set_flag(&Flag6502::B, false);
    self.set_flag(&Flag6502::U, false);
    self.stack_pointer_decrement();
    0
  }

  /// Pull accumulator
  pub fn pla(&mut self) -> u8 {
    self.stack_pointer_increment();
    self.acc = self.bus_mut_read_u8(self.get_stack_address());
    self.set_flags_zero_and_negative(self.acc.into());
    0
  }

  /// Pull processor status (SR)
  pub fn plp(&mut self) -> u8 {
    self.stack_pointer_increment();
    self.status_register = self.bus_mut_read_u8(self.get_stack_address());
    self.set_flag(&Flag6502::U, true);
    0
  }

  /// Rotate left
  pub fn rol(&mut self) -> u8 {
    self.fetch();
    let val = (u16::try_from(self.fetched).unwrap() << 1) | self.get_flag_val(&Flag6502::C);
    self.set_flag(&Flag6502::C, (val & 0xFF00) > 0);
    self.set_flags_zero_and_negative(val);

    self.return_or_write_memory(val);
    0
  }

  /// Rotate right
  pub fn ror(&mut self) -> u8 {
    self.fetch();
    let val = (self.get_flag_val(&Flag6502::C) << 7) | (u16::try_from(self.fetched).unwrap() >> 1);
    self.set_flag(&Flag6502::C, (self.fetched & 0x01) > 0);
    self.set_flags_zero_and_negative(val);

    self.return_or_write_memory(val);
    0
  }

  fn addr_mode(&mut self) -> AddrMode6502 {
    let idx = usize::try_from(self.opcode).unwrap();
    *self.lookup.get_addr_mode(idx)
  }

  /// Return form interrupt
  pub fn rti(&mut self) -> u8 {
    self.stack_pointer_increment();
    self.status_register = self.bus_mut_read_u8(self.get_stack_address());
    self.status_register &= !Flag6502::B.value();
    self.status_register &= !Flag6502::U.value();

    self.stack_pointer_increment();
    self.pc = self.bus_mut_read_u8(self.get_stack_address()) as u16;
    self.stack_pointer_increment();
    self.pc |= u16::try_from(self.bus_mut_read_u8(self.get_stack_address())).unwrap().wrapping_shl(8);
    0
  }

  /// Return form subroutine
  pub fn rts(&mut self) -> u8 {
    self.stack_pointer_increment();
    self.pc = self.bus_mut_read_u8(self.get_stack_address()) as u16;

    self.stack_pointer_increment();
    self.pc |= u16::try_from(self.bus_mut_read_u8(self.get_stack_address())).unwrap().wrapping_shl(8);

    self.pc_increment();
    0
  }

  /// Subtract with carry
  pub fn sbc(&mut self) -> u8 {
    self.fetch();
    let value = u16::try_from(self.fetched).unwrap() ^ 0xFF;

    let val = u16::try_from(self.acc).unwrap()
      .wrapping_add(value)
      .wrapping_add(self.get_flag_val(&Flag6502::C));

    self.set_flag(&Flag6502::C, (val & 0xFF00) > 0);
    self.set_flag(&Flag6502::V, ((val ^ u16::try_from(self.acc).unwrap()) & (val ^ value) & 0x80) > 0);
    self.set_flags_zero_and_negative(val & 0xFF);
    self.acc = u8::try_from(val & 0xFF).unwrap();
    1
  }

  /// Set carry
  pub fn sec(&mut self) -> u8 {
    self.set_flag(&Flag6502::C, true);
    0
  }

  /// Set decimal
  pub fn sed(&mut self) -> u8 {
    self.set_flag(&Flag6502::D, true);
    0
  }

  /// Set interrupt disable
  pub fn sei(&mut self) -> u8 {
    self.set_flag(&Flag6502::I, true);
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
    self.set_flags_zero_and_negative(self.x.into());
    0
  }

  /// Transfer accumulator to Y
  pub fn tay(&mut self) -> u8 {
    self.y = self.acc;
    self.set_flags_zero_and_negative(self.y.into());
    0
  }

  /// Transfer stack pointer to X
  pub fn tsx(&mut self) -> u8 {
    self.x = self.stack_pointer;
    self.set_flags_zero_and_negative(self.x.into());
    0
  }

  /// Transfer X to accumulator
  pub fn txa(&mut self) -> u8 {
    self.acc = self.x;
    self.set_flags_zero_and_negative(self.acc.into());
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
    self.set_flags_zero_and_negative(self.acc.into());
    0
  }

//   #[allow(dead_code)]
//   pub fn disassemble(&mut self, start: u16, end: u16) -> HashMap<u16, String> {
//     let mut addr = start as u32;
//     let mut map: HashMap<u16, String> = HashMap::new();
//
//     while addr < end as u32 {
//       let line_addr = u16::try_from(addr).unwrap();
//       let mut codes = format!("$:{}: ", hex(usize::try_from(addr).unwrap(), 4));
//       let opcode = self.bus.read_u8(u16::try_from(addr).unwrap());
//       addr += 1;
//
//       let name = self.lookup.get_name(opcode.try_into().unwrap());
//       codes = format!("{} {} ", codes, name);
//
//       let addr_mode = *self
//         .lookup
//         .get_addr_mode(opcode.try_into().unwrap());
//
//       match addr_mode {
//         AddrMode6502::Imp => {
//           codes.push_str(" {{IMP}}\t");
//         }
//         AddrMode6502::Imm => {
//           let value = self.bus_mut_read_u8(addr.try_into().unwrap());
//           addr += 1;
//           codes.push_str(&format!("${} {{IMM}}\t", hex(usize::from(value), 2)));
//         }
//         AddrMode6502::Zpo => {
//           let lo_byte = self.bus_mut_read_u8(u16::try_from(addr).unwrap());
//           addr += 1;
//           codes.push_str(&format!("${} {{ZPO}}\t", hex(usize::from(lo_byte), 2)));
//         }
//         AddrMode6502::Zpx => {
//           let lo_byte = self.bus_mut_read_u8(addr.try_into().unwrap());
//           addr += 1;
//           codes.push_str(&format!("${} {{ZPX}}\t", hex(usize::from(lo_byte), 2)));
//         }
//         AddrMode6502::Zpy => {
//           let lo_byte = self.bus_mut_read_u8(addr.try_into().unwrap());
//           addr += 1;
//           codes.push_str(&format!("${} {{ZPY}}\t", hex(usize::from(lo_byte), 2)));
//         }
//         AddrMode6502::Rel => {
//           let value = self.bus_mut_read_u8(addr.try_into().unwrap());
//           addr += 1;
//           codes.push_str(&format!(
//             "${} [${}] {{REL}}\t",
//             hex(usize::from(value), 2),
//             hex((addr.wrapping_add(value.into())).try_into().unwrap(), 4)
//           ));
//         }
//         AddrMode6502::Abs => {
//           let (lo_byte, hi_byte) = self.extract_addr_16(addr);
//           codes.push_str(&format!(
//             "${} {{ABS}}\t",
//             hex(usize::from(hi_byte.wrapping_shl(8) | lo_byte), 4)
//           ));
//         }
//         AddrMode6502::Abx => {
//           let (lo_byte, hi_byte) = self.extract_addr_16(addr);
//           codes.push_str(&format!(
//             "${} X {{ABX}}\t",
//             hex(usize::from(hi_byte.wrapping_shl(8) | lo_byte), 4)
//           ));
//         }
//         AddrMode6502::Aby => {
//           let (lo_byte, hi_byte) = self.extract_addr_16(addr);
//           codes.push_str(&format!(
//             "${}, Y {{ABY}}\t",
//             hex(usize::from(hi_byte.wrapping_shl(8) | lo_byte), 4)
//           ));
//         }
//         AddrMode6502::Ind => {
//           let (lo_byte, hi_byte) = self.extract_addr_16(addr);
//           codes.push_str(&format!(
//             "(${}) {{IND}}\t",
//             hex(usize::from(hi_byte.wrapping_shl(8) | lo_byte), 4)
//           ));
//         }
//         AddrMode6502::Izx => {
//           let lo_byte = self.bus_mut_read_u8(addr.try_into().unwrap());
//           addr += 1;
//           codes.push_str(&format!("${} {{IZX}}\t", hex(usize::from(lo_byte), 2)));
//         }
//         AddrMode6502::Izy => {
//           let lo_byte = self.bus_mut_read_u8(addr.try_into().unwrap());
//           addr += 1;
//           codes.push_str(&format!("${} {{IZY}}\t", hex(usize::from(lo_byte), 2)));
//         }
//       }
//
//       map.insert(line_addr, codes);
//     }
//     map
//   }
//
//   #[allow(dead_code)]
//   fn extract_addr_16(&mut self, mut addr: u32) -> (u16, u16) {
//     let lo_byte = self.bus_mut_read_u8(addr.try_into().unwrap());
//     addr += 1;
//     let hi_byte = self.bus_mut_read_u8(addr.try_into().unwrap());
//     (lo_byte, hi_byte)
//   }
}

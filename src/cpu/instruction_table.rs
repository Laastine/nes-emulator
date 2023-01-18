use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Eq, PartialEq)]
pub enum Flag6502 {
  C,
  Z,
  I,
  D,
  B,
  U,
  V,
  N,
}

impl Flag6502 {
  pub fn value(&self) -> u8 {
    match *self {
      Flag6502::C => 1,   // Carry
      Flag6502::Z => 2,   // Zero
      Flag6502::I => 4,   // Disable Interrupts
      Flag6502::D => 8,   // Decimal Mode
      Flag6502::B => 16,  // Break
      Flag6502::U => 32,  // Push
      Flag6502::V => 64,  // Overflow
      Flag6502::N => 128, // Negative
    }
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OpCode6502 {
  Adc,
  And,
  Asl,
  Bcc,
  Bcs,
  Beq,
  Bit,
  Bmi,
  Bne,
  Bpl,
  Brk,
  Bvc,
  Bvs,
  Clc,
  Cld,
  Cli,
  Clv,
  Cmp,
  Cpx,
  Cpy,
  Dec,
  Dex,
  Dey,
  Eor,
  Inc,
  Inx,
  Iny,
  Jmp,
  Jsr,
  Lda,
  Ldx,
  Ldy,
  Lsr,
  Nop,
  Ora,
  Pha,
  Php,
  Pla,
  Plp,
  Rol,
  Ror,
  Rti,
  Rts,
  Sbc,
  Sec,
  Sed,
  Sei,
  Sta,
  Stx,
  Sty,
  Tax,
  Tay,
  Tsx,
  Txa,
  Txs,
  Tya,
  Xxx,
}

impl fmt::Display for OpCode6502 {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      OpCode6502::Adc => write!(f, "adc"),
      OpCode6502::And => write!(f, "and"),
      OpCode6502::Asl => write!(f, "asl"),
      OpCode6502::Bcc => write!(f, "bcc"),
      OpCode6502::Bcs => write!(f, "bcs"),
      OpCode6502::Beq => write!(f, "beq"),
      OpCode6502::Bit => write!(f, "bit"),
      OpCode6502::Bmi => write!(f, "bmi"),
      OpCode6502::Bne => write!(f, "bne"),
      OpCode6502::Bpl => write!(f, "bpl"),
      OpCode6502::Brk => write!(f, "brk"),
      OpCode6502::Bvc => write!(f, "bvc"),
      OpCode6502::Bvs => write!(f, "bvs"),
      OpCode6502::Clc => write!(f, "clc"),
      OpCode6502::Cld => write!(f, "cld"),
      OpCode6502::Cli => write!(f, "cli"),
      OpCode6502::Clv => write!(f, "clv"),
      OpCode6502::Cmp => write!(f, "cmp"),
      OpCode6502::Cpx => write!(f, "cpx"),
      OpCode6502::Cpy => write!(f, "cpy"),
      OpCode6502::Dec => write!(f, "dec"),
      OpCode6502::Dex => write!(f, "dex"),
      OpCode6502::Dey => write!(f, "dey"),
      OpCode6502::Eor => write!(f, "eor"),
      OpCode6502::Inc => write!(f, "inc"),
      OpCode6502::Inx => write!(f, "inx"),
      OpCode6502::Iny => write!(f, "iny"),
      OpCode6502::Jmp => write!(f, "jmp"),
      OpCode6502::Jsr => write!(f, "jsr"),
      OpCode6502::Lda => write!(f, "lda"),
      OpCode6502::Ldx => write!(f, "ldx"),
      OpCode6502::Ldy => write!(f, "ldy"),
      OpCode6502::Lsr => write!(f, "lsr"),
      OpCode6502::Nop => write!(f, "nop"),
      OpCode6502::Ora => write!(f, "ora"),
      OpCode6502::Pha => write!(f, "pha"),
      OpCode6502::Php => write!(f, "php"),
      OpCode6502::Pla => write!(f, "pla"),
      OpCode6502::Plp => write!(f, "plp"),
      OpCode6502::Rol => write!(f, "rol"),
      OpCode6502::Ror => write!(f, "ror"),
      OpCode6502::Rti => write!(f, "rti"),
      OpCode6502::Rts => write!(f, "rts"),
      OpCode6502::Sbc => write!(f, "sbc"),
      OpCode6502::Sec => write!(f, "sec"),
      OpCode6502::Sed => write!(f, "sed"),
      OpCode6502::Sei => write!(f, "sei"),
      OpCode6502::Sta => write!(f, "sta"),
      OpCode6502::Stx => write!(f, "stx"),
      OpCode6502::Sty => write!(f, "sty"),
      OpCode6502::Tax => write!(f, "tax"),
      OpCode6502::Tay => write!(f, "tay"),
      OpCode6502::Tsx => write!(f, "tsx"),
      OpCode6502::Txa => write!(f, "txa"),
      OpCode6502::Txs => write!(f, "txs"),
      OpCode6502::Tya => write!(f, "tya"),
      OpCode6502::Xxx => write!(f, "xxx"),
    }
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AddrMode6502 {
  Abs,
  Abx,
  Aby,
  Imm,
  Imp,
  Ind,
  Izx,
  Izy,
  Rel,
  Zpo,
  Zpx,
  Zpy,
}

#[derive(Copy, Clone, Debug)]
pub struct Instruction6502 {
  pub operate: OpCode6502,
  pub addr_mode: AddrMode6502,
  pub cycles: u8,
  pub extra_cycles: u8,
}

impl Instruction6502 {
  pub fn new(
    operate: OpCode6502,
    addr_mode: AddrMode6502,
    cycles: u8,
    extra_cycles: u8,
  ) -> Instruction6502 {
    Instruction6502 {
      operate,
      addr_mode,
      cycles,
      extra_cycles,
    }
  }
}

pub struct LookUpTable {
  pub instructions: [Instruction6502; 256],
}

impl LookUpTable {
  pub fn new() -> LookUpTable {
    let instructions = [
      Instruction6502::new(OpCode6502::Brk, AddrMode6502::Imp, 7, 0),
      Instruction6502::new(OpCode6502::Ora, AddrMode6502::Izx, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izx, 8, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Ora, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Asl, AddrMode6502::Zpo, 5, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpo, 5, 0),
      Instruction6502::new(OpCode6502::Php, AddrMode6502::Imp, 3, 0),
      Instruction6502::new(OpCode6502::Ora, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Asl, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Ora, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Asl, AddrMode6502::Abs, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abs, 6, 0),

      // 0x10
      Instruction6502::new(OpCode6502::Bpl, AddrMode6502::Rel, 2, 1),
      Instruction6502::new(OpCode6502::Ora, AddrMode6502::Izy, 5, 1),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izy, 8, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Ora, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Asl, AddrMode6502::Zpx, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpx, 6, 0),
      Instruction6502::new(OpCode6502::Clc, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Ora, AddrMode6502::Aby, 4, 1),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Aby, 7, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Ora, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Asl, AddrMode6502::Abx, 7, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abx, 7, 0),

      // 0x20
      Instruction6502::new(OpCode6502::Jsr, AddrMode6502::Abs, 6, 0),
      Instruction6502::new(OpCode6502::And, AddrMode6502::Izx, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izx, 8, 0),
      Instruction6502::new(OpCode6502::Bit, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::And, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Rol, AddrMode6502::Zpo, 5, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpo, 5, 0),
      Instruction6502::new(OpCode6502::Plp, AddrMode6502::Imp, 4, 0),
      Instruction6502::new(OpCode6502::And, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Rol, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Bit, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::And, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Rol, AddrMode6502::Abs, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abs, 6, 0),

      // 0x30
      Instruction6502::new(OpCode6502::Bmi, AddrMode6502::Rel, 2, 1),
      Instruction6502::new(OpCode6502::And, AddrMode6502::Izy, 5, 1),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izy, 8, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::And, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Rol, AddrMode6502::Zpx, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpx, 6, 0),
      Instruction6502::new(OpCode6502::Sec, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::And, AddrMode6502::Aby, 4, 1),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Aby, 7, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::And, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Rol, AddrMode6502::Abx, 7, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abx, 7, 0),

      // 0x40
      Instruction6502::new(OpCode6502::Rti, AddrMode6502::Imp, 6, 0),
      Instruction6502::new(OpCode6502::Eor, AddrMode6502::Izx, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izx, 8, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Eor, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Lsr, AddrMode6502::Zpo, 5, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpo, 5, 0),
      Instruction6502::new(OpCode6502::Pha, AddrMode6502::Imp, 3, 0),
      Instruction6502::new(OpCode6502::Eor, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Lsr, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Jmp, AddrMode6502::Abs, 3, 0),
      Instruction6502::new(OpCode6502::Eor, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Lsr, AddrMode6502::Abs, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abs, 6, 0),

      // 0x50
      Instruction6502::new(OpCode6502::Bvc, AddrMode6502::Rel, 2, 1),
      Instruction6502::new(OpCode6502::Eor, AddrMode6502::Izy, 5, 1),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izy, 8, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Eor, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Lsr, AddrMode6502::Zpx, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpx, 6, 0),
      Instruction6502::new(OpCode6502::Cli, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Eor, AddrMode6502::Aby, 4, 1),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Aby, 7, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Eor, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Lsr, AddrMode6502::Abx, 7, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abx, 7, 0),

      // 0x60
      Instruction6502::new(OpCode6502::Rts, AddrMode6502::Imp, 6, 0),
      Instruction6502::new(OpCode6502::Adc, AddrMode6502::Izx, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izx, 8, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Adc, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Ror, AddrMode6502::Zpo, 5, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpo, 5, 0),
      Instruction6502::new(OpCode6502::Pla, AddrMode6502::Imp, 4, 0),
      Instruction6502::new(OpCode6502::Adc, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Ror, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Jmp, AddrMode6502::Ind, 5, 0),
      Instruction6502::new(OpCode6502::Adc, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Ror, AddrMode6502::Abs, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abs, 6, 0),

      // 0x70
      Instruction6502::new(OpCode6502::Bvs, AddrMode6502::Rel, 2, 1),
      Instruction6502::new(OpCode6502::Adc, AddrMode6502::Izy, 5, 1),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izy, 8, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Adc, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Ror, AddrMode6502::Zpx, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpx, 6, 0),
      Instruction6502::new(OpCode6502::Sei, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Adc, AddrMode6502::Aby, 4, 1),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Aby, 7, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Adc, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Ror, AddrMode6502::Abx, 7, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abx, 7, 0),

      // 0x80
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Sta, AddrMode6502::Izx, 6, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izx, 6, 0),
      Instruction6502::new(OpCode6502::Sty, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Sta, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Stx, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Dey, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Txa, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Sty, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Sta, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Stx, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abs, 4, 0),

      // 0x90
      Instruction6502::new(OpCode6502::Bcc, AddrMode6502::Rel, 2, 1),
      Instruction6502::new(OpCode6502::Sta, AddrMode6502::Izy, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Sty, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Sta, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Stx, AddrMode6502::Zpy, 4, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpy, 4, 0),
      Instruction6502::new(OpCode6502::Tya, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Sta, AddrMode6502::Aby, 5, 0),
      Instruction6502::new(OpCode6502::Txs, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Sta, AddrMode6502::Abx, 5, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),

      // 0xA0
      Instruction6502::new(OpCode6502::Ldy, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Lda, AddrMode6502::Izx, 6, 0),
      Instruction6502::new(OpCode6502::Ldx, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izx, 6, 0),
      Instruction6502::new(OpCode6502::Ldy, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Lda, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Ldx, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Tay, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Lda, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Tax, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Ldy, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Lda, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Ldx, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abs, 4, 0),

      // 0xB0
      Instruction6502::new(OpCode6502::Bcs, AddrMode6502::Rel, 2, 1),
      Instruction6502::new(OpCode6502::Lda, AddrMode6502::Izy, 5, 1),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izy, 5, 1),
      Instruction6502::new(OpCode6502::Ldy, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Lda, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Ldx, AddrMode6502::Zpy, 4, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpy, 4, 0),
      Instruction6502::new(OpCode6502::Clv, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Lda, AddrMode6502::Aby, 4, 1),
      Instruction6502::new(OpCode6502::Tsx, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Ldy, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Lda, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Ldx, AddrMode6502::Aby, 4, 1),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Aby, 4, 1),

      // 0xC0
      Instruction6502::new(OpCode6502::Cpy, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Cmp, AddrMode6502::Izx, 6, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izx, 8, 0),
      Instruction6502::new(OpCode6502::Cpy, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Cmp, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Dec, AddrMode6502::Zpo, 5, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpo, 5, 0),
      Instruction6502::new(OpCode6502::Iny, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Cmp, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Dex, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Cpy, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Cmp, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Dec, AddrMode6502::Abs, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abs, 6, 0),

      // 0xD0
      Instruction6502::new(OpCode6502::Bne, AddrMode6502::Rel, 2, 1),
      Instruction6502::new(OpCode6502::Cmp, AddrMode6502::Izy, 5, 1),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izy, 8, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Cmp, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Dec, AddrMode6502::Zpx, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpx, 6, 0),
      Instruction6502::new(OpCode6502::Cld, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Cmp, AddrMode6502::Aby, 4, 1),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Aby, 7, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Cmp, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Dec, AddrMode6502::Abx, 7, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abx, 7, 0),

      // 0xE0
      Instruction6502::new(OpCode6502::Cpx, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Sbc, AddrMode6502::Izx, 6, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izx, 8, 0),
      Instruction6502::new(OpCode6502::Cpx, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Sbc, AddrMode6502::Zpo, 3, 0),
      Instruction6502::new(OpCode6502::Inc, AddrMode6502::Zpo, 5, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpo, 5, 0),
      Instruction6502::new(OpCode6502::Inx, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Sbc, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Sbc, AddrMode6502::Imm, 2, 0),
      Instruction6502::new(OpCode6502::Cpx, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Sbc, AddrMode6502::Abs, 4, 0),
      Instruction6502::new(OpCode6502::Inc, AddrMode6502::Abs, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abs, 6, 0),

      // 0xF0
      Instruction6502::new(OpCode6502::Beq, AddrMode6502::Rel, 2, 1),
      Instruction6502::new(OpCode6502::Sbc, AddrMode6502::Izy, 5, 1),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Imp, 0, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Izy, 8, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Sbc, AddrMode6502::Zpx, 4, 0),
      Instruction6502::new(OpCode6502::Inc, AddrMode6502::Zpx, 6, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Zpx, 6, 0),
      Instruction6502::new(OpCode6502::Sed, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Sbc, AddrMode6502::Aby, 4, 1),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Imp, 2, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Aby, 7, 0),
      Instruction6502::new(OpCode6502::Nop, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Sbc, AddrMode6502::Abx, 4, 1),
      Instruction6502::new(OpCode6502::Inc, AddrMode6502::Abx, 7, 0),
      Instruction6502::new(OpCode6502::Xxx, AddrMode6502::Abx, 7, 0),
    ];

    LookUpTable { instructions }
  }

  #[allow(dead_code)]
  pub fn get_name(&self, index: usize) -> String {
    format!("{:?}", &self.instructions[index])
  }

  pub fn get_addr_mode(&self, index: usize) -> &AddrMode6502 {
    &self.instructions[index].addr_mode
  }

  pub fn get_operate(&self, index: usize) -> &OpCode6502 {
    &self.instructions[index].operate
  }

  pub fn get_cycles(&self, index: usize) -> u8 { self.instructions[index].cycles }
}

pub fn hex(num: usize, len: usize) -> String {
  match len {
    2 => format!("{:0>2X}", num),
    4 => format!("{:0>4X}", num),
    _ => panic!("Unknown length"),
  }
}

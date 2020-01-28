#[derive(Debug, PartialEq)]
pub enum FLAGS6502 {
  C,
  Z,
  I,
  D,
  B,
  U,
  V,
  N,
}

impl FLAGS6502 {
  pub fn value(&self) -> u8 {
    match *self {
      FLAGS6502::C => 1,   // Carry
      FLAGS6502::Z => 2,   // Zero
      FLAGS6502::I => 4,   // Disable Interrupts
      FLAGS6502::D => 8,   // Decimal Mode
      FLAGS6502::B => 16,  // Break
      FLAGS6502::U => 32,  // Unused
      FLAGS6502::V => 64,  // Overflow
      FLAGS6502::N => 128, // Negative
    }
  }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OPCODES6502 {
  ADC,
  AND,
  ASL,
  BCC,
  BCS,
  BEQ,
  BIT,
  BMI,
  BNE,
  BPL,
  BRK,
  BVC,
  BVS,
  CLC,
  CLD,
  CLI,
  CLV,
  CMP,
  CPX,
  CPY,
  DEC,
  DEX,
  DEY,
  EOR,
  INC,
  INX,
  INY,
  JMP,
  JSR,
  LDA,
  LDX,
  LDY,
  LSR,
  NOP,
  ORA,
  PHA,
  PHP,
  PLA,
  PLP,
  ROL,
  ROR,
  RTI,
  RTS,
  SBC,
  SEC,
  SED,
  SEI,
  STA,
  STX,
  STY,
  TAX,
  TAY,
  TSX,
  TXA,
  TXS,
  TYA,
  XXX,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ADDRMODE6502 {
  IMP,
  IMM,
  ZP0,
  ZPX,
  ZPY,
  REL,
  ABS,
  ABX,
  ABY,
  IND,
  IZX,
  IZY,
}

#[derive(Copy, Clone, Debug)]
pub struct Instruction6502 {
  pub operate: OPCODES6502,
  pub addr_mode: ADDRMODE6502,
  pub cycles: u8,
}

impl Instruction6502 {
  pub fn new(
    operate: OPCODES6502,
    addr_mode: ADDRMODE6502,
    cycles: u8,
  ) -> Instruction6502 {
    Instruction6502 {
      operate,
      addr_mode,
      cycles,
    }
  }
}

pub struct LookUpTable {
  pub instructions: Vec<Instruction6502>,
}

impl LookUpTable {
  pub fn new() -> LookUpTable {
    let mut instructions: Vec<Instruction6502> = Vec::with_capacity(256);

    instructions.push(Instruction6502::new(OPCODES6502::BRK, ADDRMODE6502::IMM, 7));
    instructions.push(Instruction6502::new(OPCODES6502::ORA, ADDRMODE6502::IZX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 8));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 3));
    instructions.push(Instruction6502::new(OPCODES6502::ORA, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::ASL, ADDRMODE6502::ZP0, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 5));
    instructions.push(Instruction6502::new(OPCODES6502::PHP, ADDRMODE6502::IMP, 3));
    instructions.push(Instruction6502::new(OPCODES6502::ORA, ADDRMODE6502::IMM, 2));
    instructions.push(Instruction6502::new(OPCODES6502::ASL, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ORA, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ASL, ADDRMODE6502::ABS, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::BPL, ADDRMODE6502::REL, 2));
    instructions.push(Instruction6502::new(OPCODES6502::ORA, ADDRMODE6502::IZY, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 8));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ORA, ADDRMODE6502::ZPX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ASL, ADDRMODE6502::ZPX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::CLC, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::ORA, ADDRMODE6502::ABY, 4));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 7));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ORA, ADDRMODE6502::ABX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ASL, ADDRMODE6502::ABX, 7));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 7));
    instructions.push(Instruction6502::new(OPCODES6502::JSR, ADDRMODE6502::ABS, 6));
    instructions.push(Instruction6502::new(OPCODES6502::AND, ADDRMODE6502::IZX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 8));
    instructions.push(Instruction6502::new(OPCODES6502::BIT, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::AND, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::ROL, ADDRMODE6502::ZP0, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 5));
    instructions.push(Instruction6502::new(OPCODES6502::PLP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::AND, ADDRMODE6502::IMM, 2));
    instructions.push(Instruction6502::new(OPCODES6502::ROL, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::BIT, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::AND, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ROL, ADDRMODE6502::ABS, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::BMI, ADDRMODE6502::REL, 2));
    instructions.push(Instruction6502::new(OPCODES6502::AND, ADDRMODE6502::IZY, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 8));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::AND, ADDRMODE6502::ZPX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ROL, ADDRMODE6502::ZPX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::SEC, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::AND, ADDRMODE6502::ABY, 4));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 7));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::AND, ADDRMODE6502::ABX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ROL, ADDRMODE6502::ABX, 7));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 7));
    instructions.push(Instruction6502::new(OPCODES6502::RTI, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::EOR, ADDRMODE6502::IZX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 8));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 3));
    instructions.push(Instruction6502::new(OPCODES6502::EOR, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::LSR, ADDRMODE6502::ZP0, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 5));
    instructions.push(Instruction6502::new(OPCODES6502::PHA, ADDRMODE6502::IMP, 3));
    instructions.push(Instruction6502::new(OPCODES6502::EOR, ADDRMODE6502::IMM, 2));
    instructions.push(Instruction6502::new(OPCODES6502::LSR, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::JMP, ADDRMODE6502::ABS, 3));
    instructions.push(Instruction6502::new(OPCODES6502::EOR, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::LSR, ADDRMODE6502::ABS, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::BVC, ADDRMODE6502::REL, 2));
    instructions.push(Instruction6502::new(OPCODES6502::EOR, ADDRMODE6502::IZY, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 8));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::EOR, ADDRMODE6502::ZPX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::LSR, ADDRMODE6502::ZPX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::CLI, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::EOR, ADDRMODE6502::ABY, 4));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 7));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::EOR, ADDRMODE6502::ABX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::LSR, ADDRMODE6502::ABX, 7));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 7));
    instructions.push(Instruction6502::new(OPCODES6502::RTS, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::ADC, ADDRMODE6502::IZX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 8));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 3));
    instructions.push(Instruction6502::new(OPCODES6502::ADC, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::ROR, ADDRMODE6502::ZP0, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 5));
    instructions.push(Instruction6502::new(OPCODES6502::PLA, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ADC, ADDRMODE6502::IMM, 2));
    instructions.push(Instruction6502::new(OPCODES6502::ROR, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::JMP, ADDRMODE6502::IND, 5));
    instructions.push(Instruction6502::new(OPCODES6502::ADC, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ROR, ADDRMODE6502::ABS, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::BVS, ADDRMODE6502::REL, 2));
    instructions.push(Instruction6502::new(OPCODES6502::ADC, ADDRMODE6502::IZY, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 8));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ADC, ADDRMODE6502::ZPX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ROR, ADDRMODE6502::ZPX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::SEI, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::ADC, ADDRMODE6502::ABY, 4));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 7));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ADC, ADDRMODE6502::ABX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::ROR, ADDRMODE6502::ABX, 7));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 7));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::STA, ADDRMODE6502::IZX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::STY, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::STA, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::STX, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 3));
    instructions.push(Instruction6502::new(OPCODES6502::DEY, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::TXA, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::STY, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::STA, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::STX, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::BCC, ADDRMODE6502::REL, 2));
    instructions.push(Instruction6502::new(OPCODES6502::STA, ADDRMODE6502::IZY, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::STY, ADDRMODE6502::ZPX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::STA, ADDRMODE6502::ZPX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::STX, ADDRMODE6502::ZPY, 4));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::TYA, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::STA, ADDRMODE6502::ABY, 5));
    instructions.push(Instruction6502::new(OPCODES6502::TXS, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 5));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 5));
    instructions.push(Instruction6502::new(OPCODES6502::STA, ADDRMODE6502::ABX, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 5));
    instructions.push(Instruction6502::new(OPCODES6502::LDY, ADDRMODE6502::IMM, 2));
    instructions.push(Instruction6502::new(OPCODES6502::LDA, ADDRMODE6502::IZX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::LDX, ADDRMODE6502::IMM, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::LDY, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::LDA, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::LDX, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 3));
    instructions.push(Instruction6502::new(OPCODES6502::TAY, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::LDA, ADDRMODE6502::IMM, 2));
    instructions.push(Instruction6502::new(OPCODES6502::TAX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::LDY, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::LDA, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::LDX, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::BCS, ADDRMODE6502::REL, 2));
    instructions.push(Instruction6502::new(OPCODES6502::LDA, ADDRMODE6502::IZY, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 5));
    instructions.push(Instruction6502::new(OPCODES6502::LDY, ADDRMODE6502::ZPX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::LDA, ADDRMODE6502::ZPX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::LDX, ADDRMODE6502::ZPY, 4));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::CLV, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::LDA, ADDRMODE6502::ABY, 4));
    instructions.push(Instruction6502::new(OPCODES6502::TSX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::LDY, ADDRMODE6502::ABX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::LDA, ADDRMODE6502::ABX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::LDX, ADDRMODE6502::ABY, 4));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::CPY, ADDRMODE6502::IMM, 2));
    instructions.push(Instruction6502::new(OPCODES6502::CMP, ADDRMODE6502::IZX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 8));
    instructions.push(Instruction6502::new(OPCODES6502::CPY, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::CMP, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::DEC, ADDRMODE6502::ZP0, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 5));
    instructions.push(Instruction6502::new(OPCODES6502::INY, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::CMP, ADDRMODE6502::IMM, 2));
    instructions.push(Instruction6502::new(OPCODES6502::DEX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::CPY, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::CMP, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::DEC, ADDRMODE6502::ABS, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::BNE, ADDRMODE6502::REL, 2));
    instructions.push(Instruction6502::new(OPCODES6502::CMP, ADDRMODE6502::IZY, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 8));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::CMP, ADDRMODE6502::ZPX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::DEC, ADDRMODE6502::ZPX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::CLD, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::CMP, ADDRMODE6502::ABY, 4));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 7));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::CMP, ADDRMODE6502::ABX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::DEC, ADDRMODE6502::ABX, 7));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 7));
    instructions.push(Instruction6502::new(OPCODES6502::CPX, ADDRMODE6502::IMM, 2));
    instructions.push(Instruction6502::new(OPCODES6502::SBC, ADDRMODE6502::IZX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 8));
    instructions.push(Instruction6502::new(OPCODES6502::CPX, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::SBC, ADDRMODE6502::ZP0, 3));
    instructions.push(Instruction6502::new(OPCODES6502::INC, ADDRMODE6502::ZP0, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 5));
    instructions.push(Instruction6502::new(OPCODES6502::INX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::SBC, ADDRMODE6502::IMM, 2));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::SBC, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::CPX, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::SBC, ADDRMODE6502::ABS, 4));
    instructions.push(Instruction6502::new(OPCODES6502::INC, ADDRMODE6502::ABS, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::BEQ, ADDRMODE6502::REL, 2));
    instructions.push(Instruction6502::new(OPCODES6502::SBC, ADDRMODE6502::IZY, 5));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 8));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::SBC, ADDRMODE6502::ZPX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::INC, ADDRMODE6502::ZPX, 6));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 6));
    instructions.push(Instruction6502::new(OPCODES6502::SED, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::SBC, ADDRMODE6502::ABY, 4));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 2));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 7));
    instructions.push(Instruction6502::new(OPCODES6502::NOP, ADDRMODE6502::IMP, 4));
    instructions.push(Instruction6502::new(OPCODES6502::SBC, ADDRMODE6502::ABX, 4));
    instructions.push(Instruction6502::new(OPCODES6502::INC, ADDRMODE6502::ABX, 7));
    instructions.push(Instruction6502::new(OPCODES6502::XXX, ADDRMODE6502::IMP, 7));

    LookUpTable { instructions }
  }

  pub fn get_addr_mode(&self, index: usize) -> &ADDRMODE6502 {
    &self.instructions[index].addr_mode
  }

  pub fn get_operate(&self, index: usize) -> &OPCODES6502 {
    &self.instructions[index].operate
  }

  #[cfg(feature = "terminal_debug")]
  pub fn get_name(&self, index: usize) -> String {
    format!("{:?}", self.instructions[index].operate)
  }

  pub fn get_cycles(&self, index: usize) -> u8 { self.instructions[index].cycles }
}

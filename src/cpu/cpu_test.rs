use std::cell::RefCell;
use std::rc::Rc;
use crate::apu::Apu;
use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::cpu::instruction_table::AddrMode6502;
use crate::cpu::instruction_table::AddrMode6502::*;
use crate::nes::controller::Controller;
use crate::ppu::registers::Registers;

macro_rules! build_cpu_and_memory {
    ($bytes: expr) => {
      {
        let cartridge = Cartridge::mock_cartridge();
        let cart = Rc::new(RefCell::new(Box::new(cartridge)));

        let registers = Rc::new(RefCell::new(Registers::new(cart.clone())));

        let controller = Rc::new(RefCell::new(Controller::new()));

        let apu = Rc::new(RefCell::new(Apu::new()));

        let bus = Bus::new(cart, registers.clone(), controller.clone(), apu.clone());

        let mut cpu = Cpu::new(bus);

        let bytes = $bytes;
        for (idx, &b) in bytes.iter().enumerate() {
          cpu.bus.ram[idx] = b as u8;
        }

        cpu
      }
    }
}

macro_rules! test_op_code {
    ($instruction:expr, $mode:ident, [$($bytes:expr),*]{$($sk:ident : $sv:expr),*} => [$($rb:expr),*]{$($ek:ident : $ev:expr),*}) => {
      {
        let op = opcode($instruction, $mode);
        let mut mem = Vec::new();
        $(mem.push($bytes);)*
        mem.insert(0, op.code);

        let mut cpu = build_cpu_and_memory!(mem);
        let start_pc = cpu.pc;
        let start_cycles = cpu.cycle;
        $(cpu.$sk=$sv;)*
        cpu.clock(0);

        if op.size > 0 {
            assert!(op.size == (cpu.pc - start_pc), "Invalid instruction size. Expected: {} bytes, Got: {}", op.size, cpu.pc - start_pc);
        }

        if op.cycles > 0 {
          assert!(op.cycles == (cpu.cycle - start_cycles), "Invalid instruction duration. Expected: {} cycles, Got: {}", op.cycles, cpu.cycle - start_cycles);
        }

        $(
            assert!(cpu.$ek==$ev, "Incorrect Register. Expected cpu.{} to be {}, got {}", stringify!($ek), stringify!($ev), cpu.$ek);
        )*
        let mut mem = Vec::new();
        $(mem.push($rb);)*
        mem.insert(0, op.code);
        for (i, &b) in mem.iter().enumerate() {
            assert!(cpu.bus.ram[i]==b, "Incorrect Memory. Expected ram[{}] to be {}, got {}", i, b, cpu.bus.ram[i]);
        }

        cpu
      }
    }
}

#[test]
fn test_lda() {
  test_op_code!("lda", Imm, [0x00]{} => []{ acc: 0x00, stack_pointer: 0b00000010 });
  test_op_code!("lda", Imm, [0xFF]{} => []{ acc: 0xFF, stack_pointer: 0b10000000 });
  test_op_code!("lda", Imm, [0x20]{} => []{ acc: 0x20, stack_pointer: 0 });
  test_op_code!("lda", Zpo,  [0x02, 0x90]{} => []{ acc: 0x90 });
  test_op_code!("lda", Zpx, [0x02, 0, 0x90]{x:1} => []{ acc: 0x90 });
  test_op_code!("lda", Abs,  [0x04, 0, 0, 0x90]{x:1} => []{ acc: 0x90 });
  test_op_code!("lda", Abx, [0x03, 0, 0, 0x90]{x:1} => []{ acc: 0x90 });
  test_op_code!("lda", Aby, [0x03, 0, 0, 0x90]{y:1} => []{ acc: 0x90 });
  test_op_code!("lda", Izy, [0x02, 0, 0x05, 0, 0x90]{x:1} => []{ acc: 0x90 });
  test_op_code!("lda", Izx, [0x02, 0x04, 0, 0, 0x90]{y:1} => []{ acc: 0x90 });
}


#[derive(Debug, Copy, Clone)]
struct Op {
  code: u8,
  size: u16,
  cycles: u8,
  // cycles: u64,
  check: bool,
  mask: u8,
}

fn opcode(name: &str, mode: AddrMode6502) -> Op {
  match (name, mode) {
    ("adc", Imm) => Op {
      code: 0x69,
      size: 2,
      cycles: 2,
      check: false,
      mask: 0b11000011,
    },
    ("adc", Zpo) => Op {
      code: 0x65,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b11000011,
    },
    ("adc", Zpx) => Op {
      code: 0x75,
      size: 2,
      cycles: 4,
      check: false,
      mask: 0b11000011,
    },
    ("adc", Abs) => Op {
      code: 0x6D,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b11000011,
    },
    ("adc", Abx) => Op {
      code: 0x7D,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b11000011,
    },
    ("adc", Aby) => Op {
      code: 0x79,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b11000011,
    },
    ("adc", Izx) => Op { //IndX
      code: 0x61,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b11000011,
    },
    ("adc", Izy) => Op {
      code: 0x71,
      size: 2,
      cycles: 5,
      check: true,
      mask: 0b11000011,
    },
    ("and", Imm) => Op {
      code: 0x29,
      size: 2,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("and", Zpo) => Op {
      code: 0x25,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b10000010,
    },
    ("and", Zpx) => Op {
      code: 0x35,
      size: 2,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("and", Abs) => Op {
      code: 0x2D,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("and", Abx) => Op {
      code: 0x3D,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b10000010,
    },
    ("and", Aby) => Op {
      code: 0x39,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b10000010,
    },
    ("and", Izx) => Op {
      code: 0x21,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b10000010,
    },
    ("and", Izy) => Op {
      code: 0x31,
      size: 2,
      cycles: 5,
      check: true,
      mask: 0b10000010,
    },
    ("asl", Imp) => Op {
      code: 0x0A,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000011,
    },
    ("asl", Zpo) => Op {
      code: 0x06,
      size: 2,
      cycles: 5,
      check: false,
      mask: 0b10000011,
    },
    ("asl", Zpx) => Op {
      code: 0x16,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b10000011,
    },
    ("asl", Abs) => Op {
      code: 0x0E,
      size: 3,
      cycles: 6,
      check: false,
      mask: 0b10000011,
    },
    ("asl", Abx) => Op {
      code: 0x1E,
      size: 3,
      cycles: 7,
      check: false,
      mask: 0b10000011,
    },
    ("bcc", Imp) => Op {
      code: 0x90,
      size: 0,
      cycles: 0,
      check: true,
      mask: 0b00000000,
    },
    ("bcs", Imp) => Op {
      code: 0xB0,
      size: 0,
      cycles: 0,
      check: true,
      mask: 0b00000000,
    },
    ("beq", Imp) => Op {
      code: 0xF0,
      size: 0,
      cycles: 0,
      check: true,
      mask: 0b00000000,
    },
    ("bit", Zpo) => Op {
      code: 0x24,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b11000010,
    },
    ("bit", Abs) => Op {
      code: 0x2C,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b11000010,
    },
    ("bmi", Imp) => Op {
      code: 0x30,
      size: 0,
      cycles: 0,
      check: true,
      mask: 0b00000000,
    },
    ("bne", Imp) => Op {
      code: 0xD0,
      size: 0,
      cycles: 0,
      check: true,
      mask: 0b00000000,
    },
    ("bpl", Imp) => Op {
      code: 0x10,
      size: 0,
      cycles: 0,
      check: true,
      mask: 0b00000000,
    },
    ("brk", Imp) => Op {
      code: 0x00,
      size: 0,
      cycles: 7,
      check: false,
      mask: 0b00010000,
    },
    ("bvc", Imp) => Op {
      code: 0x50,
      size: 0,
      cycles: 0,
      check: true,
      mask: 0b00000000,
    },
    ("bvs", Imp) => Op {
      code: 0x70,
      size: 0,
      cycles: 0,
      check: true,
      mask: 0b00000000,
    },
    ("clc", Imp) => Op {
      code: 0x18,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b00000001,
    },
    ("cld", Imp) => Op {
      code: 0xD8,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b00001000,
    },
    ("cli", Imp) => Op {
      code: 0x58,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b00000100,
    },
    ("clv", Imp) => Op {
      code: 0xB8,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b01000000,
    },
    ("cmp", Imm) => Op {
      code: 0xC9,
      size: 2,
      cycles: 2,
      check: false,
      mask: 0b10000011,
    },
    ("cmp", Zpo) => Op {
      code: 0xC5,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b10000011,
    },
    ("cmp", Zpx) => Op {
      code: 0xD5,
      size: 2,
      cycles: 4,
      check: false,
      mask: 0b10000011,
    },
    ("cmp", Abs) => Op {
      code: 0xCD,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b10000011,
    },
    ("cmp", Abx) => Op {
      code: 0xDD,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b10000011,
    },
    ("cmp", Aby) => Op {
      code: 0xD9,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b10000011,
    },
    ("cmp", Izx) => Op {
      code: 0xC1,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b10000011,
    },
    ("cmp", Izy) => Op {
      code: 0xD1,
      size: 2,
      cycles: 5,
      check: true,
      mask: 0b10000011,
    },
    ("cpx", Imm) => Op {
      code: 0xE0,
      size: 2,
      cycles: 2,
      check: false,
      mask: 0b10000011,
    },
    ("cpx", Zpo) => Op {
      code: 0xE4,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b10000011,
    },
    ("cpx", Abs) => Op {
      code: 0xEC,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b10000011,
    },
    ("cpy", Imm) => Op {
      code: 0xC0,
      size: 2,
      cycles: 2,
      check: false,
      mask: 0b10000011,
    },
    ("cpy", Zpo) => Op {
      code: 0xC4,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b10000011,
    },
    ("cpy", Abs) => Op {
      code: 0xCC,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b10000011,
    },
    ("dec", Zpo) => Op {
      code: 0xC6,
      size: 2,
      cycles: 5,
      check: false,
      mask: 0b10000010,
    },
    ("dec", Zpx) => Op {
      code: 0xD6,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b10000010,
    },
    ("dec", Abs) => Op {
      code: 0xCE,
      size: 3,
      cycles: 6,
      check: false,
      mask: 0b10000010,
    },
    ("dec", Abx) => Op {
      code: 0xDE,
      size: 3,
      cycles: 7,
      check: false,
      mask: 0b10000010,
    },
    ("dex", Imp) => Op {
      code: 0xCA,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("dey", Imp) => Op {
      code: 0x88,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("eor", Imm) => Op {
      code: 0x49,
      size: 2,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("eor", Zpo) => Op {
      code: 0x45,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b10000010,
    },
    ("eor", Zpx) => Op {
      code: 0x55,
      size: 2,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("eor", Abs) => Op {
      code: 0x4D,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("eor", Abx) => Op {
      code: 0x5D,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b10000010,
    },
    ("eor", Aby) => Op {
      code: 0x59,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b10000010,
    },
    ("eor", Izx) => Op {
      code: 0x41,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b10000010,
    },
    ("eor", Izy) => Op {
      code: 0x51,
      size: 2,
      cycles: 5,
      check: true,
      mask: 0b10000010,
    },
    ("inc", Zpo) => Op {
      code: 0xE6,
      size: 2,
      cycles: 5,
      check: false,
      mask: 0b10000010,
    },
    ("inc", Zpx) => Op {
      code: 0xF6,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b10000010,
    },
    ("inc", Abs) => Op {
      code: 0xEE,
      size: 3,
      cycles: 6,
      check: false,
      mask: 0b10000010,
    },
    ("inc", Abx) => Op {
      code: 0xFE,
      size: 3,
      cycles: 7,
      check: false,
      mask: 0b10000010,
    },
    ("inx", Imp) => Op {
      code: 0xE8,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("iny", Imp) => Op {
      code: 0xC8,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("jmp", Abs) => Op {
      code: 0x4C,
      size: 0,
      cycles: 3,
      check: false,
      mask: 0b00000000,
    },
    ("jmp", Ind) => Op {
      code: 0x6C,
      size: 0,
      cycles: 5,
      check: false,
      mask: 0b00000000,
    },
    ("jsr", Abs) => Op {
      code: 0x20,
      size: 0,
      cycles: 6,
      check: false,
      mask: 0b00000000,
    },
    ("lda", Imm) => Op {
      code: 0xA9,
      size: 2,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("lda", Zpo) => Op {
      code: 0xA5,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b10000010,
    },
    ("lda", Zpx) => Op {
      code: 0xB5,
      size: 2,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("lda", Abs) => Op {
      code: 0xAD,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("lda", Abx) => Op {
      code: 0xBD,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b10000010,
    },
    ("lda", Aby) => Op {
      code: 0xB9,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b10000010,
    },
    ("lda", Izx) => Op {
      code: 0xA1,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b10000010,
    },
    ("lda", Izy) => Op {
      code: 0xB1,
      size: 2,
      cycles: 5,
      check: true,
      mask: 0b10000010,
    },
    ("ldx", Imm) => Op {
      code: 0xA2,
      size: 2,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("ldx", Zpo) => Op {
      code: 0xA6,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b10000010,
    },
    ("ldx", Zpy) => Op {
      code: 0xB6,
      size: 2,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("ldx", Abs) => Op {
      code: 0xAE,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("ldx", Aby) => Op {
      code: 0xBE,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b10000010,
    },
    ("ldy", Imm) => Op {
      code: 0xA0,
      size: 2,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("ldy", Zpo) => Op {
      code: 0xA4,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b10000010,
    },
    ("ldy", Zpx) => Op {
      code: 0xB4,
      size: 2,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("ldy", Abs) => Op {
      code: 0xAC,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("ldy", Abx) => Op {
      code: 0xBC,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b10000010,
    },
    ("lsr", Imp) => Op {
      code: 0x4A,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000011,
    },
    ("lsr", Zpo) => Op {
      code: 0x46,
      size: 2,
      cycles: 5,
      check: false,
      mask: 0b10000011,
    },
    ("lsr", Zpx) => Op {
      code: 0x56,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b10000011,
    },
    ("lsr", Abs) => Op {
      code: 0x4E,
      size: 3,
      cycles: 6,
      check: false,
      mask: 0b10000011,
    },
    ("lsr", Abx) => Op {
      code: 0x5E,
      size: 3,
      cycles: 7,
      check: false,
      mask: 0b10000011,
    },
    ("nop", Imp) => Op {
      code: 0xEA,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b00000000,
    },
    ("ora", Imm) => Op {
      code: 0x09,
      size: 2,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("ora", Zpo) => Op {
      code: 0x05,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b10000010,
    },
    ("ora", Zpx) => Op {
      code: 0x15,
      size: 2,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("ora", Abs) => Op {
      code: 0x0D,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("ora", Abx) => Op {
      code: 0x1D,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b10000010,
    },
    ("ora", Aby) => Op {
      code: 0x19,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b10000010,
    },
    ("ora", Izx) => Op {
      code: 0x01,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b10000010,
    },
    ("ora", Izy) => Op {
      code: 0x11,
      size: 2,
      cycles: 5,
      check: true,
      mask: 0b10000010,
    },
    ("pha", Imp) => Op {
      code: 0x48,
      size: 1,
      cycles: 3,
      check: false,
      mask: 0b00000000,
    },
    ("php", Imp) => Op {
      code: 0x08,
      size: 1,
      cycles: 3,
      check: false,
      mask: 0b00000000,
    },
    ("pla", Imp) => Op {
      code: 0x68,
      size: 1,
      cycles: 4,
      check: false,
      mask: 0b10000010,
    },
    ("plp", Imp) => Op {
      code: 0x28,
      size: 1,
      cycles: 4,
      check: false,
      mask: 0b11011111,
    },
    ("rol", Imp) => Op {
      code: 0x2A,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000011,
    },
    ("rol", Zpo) => Op {
      code: 0x26,
      size: 2,
      cycles: 5,
      check: false,
      mask: 0b10000011,
    },
    ("rol", Zpx) => Op {
      code: 0x36,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b10000011,
    },
    ("rol", Abs) => Op {
      code: 0x2E,
      size: 3,
      cycles: 6,
      check: false,
      mask: 0b10000011,
    },
    ("rol", Abx) => Op {
      code: 0x3E,
      size: 3,
      cycles: 7,
      check: false,
      mask: 0b10000011,
    },
    ("ror", Imp) => Op {
      code: 0x6A,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000011,
    },
    ("ror", Zpo) => Op {
      code: 0x66,
      size: 2,
      cycles: 5,
      check: false,
      mask: 0b10000011,
    },
    ("ror", Zpx) => Op {
      code: 0x76,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b10000011,
    },
    ("ror", Abs) => Op {
      code: 0x6E,
      size: 3,
      cycles: 6,
      check: false,
      mask: 0b10000011,
    },
    ("ror", Abx) => Op {
      code: 0x7E,
      size: 3,
      cycles: 7,
      check: false,
      mask: 0b10000011,
    },
    ("rti", Imp) => Op {
      code: 0x40,
      size: 1,
      cycles: 6,
      check: false,
      mask: 0b11011111,
    },
    ("rts", Imp) => Op {
      code: 0x60,
      size: 0,
      cycles: 6,
      check: false,
      mask: 0b00000000,
    },
    ("sbc", Imm) => Op {
      code: 0xE9,
      size: 2,
      cycles: 2,
      check: false,
      mask: 0b11000011,
    },
    ("sbc", Zpo) => Op {
      code: 0xE5,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b11000011,
    },
    ("sbc", Zpx) => Op {
      code: 0xF5,
      size: 2,
      cycles: 4,
      check: false,
      mask: 0b11000011,
    },
    ("sbc", Abs) => Op {
      code: 0xED,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b11000011,
    },
    ("sbc", Abx) => Op {
      code: 0xFD,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b11000011,
    },
    ("sbc", Aby) => Op {
      code: 0xF9,
      size: 3,
      cycles: 4,
      check: true,
      mask: 0b11000011,
    },
    ("sbc", Izx) => Op {
      code: 0xE1,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b11000011,
    },
    ("sbc", Izy) => Op {
      code: 0xF1,
      size: 2,
      cycles: 5,
      check: true,
      mask: 0b11000011,
    },
    ("sec", Imp) => Op {
      code: 0x38,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b00000001,
    },
    ("sed", Imp) => Op {
      code: 0xF8,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b00001000,
    },
    ("sei", Imp) => Op {
      code: 0x78,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b00000100,
    },
    ("sta", Zpo) => Op {
      code: 0x85,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b00000000,
    },
    ("sta", Zpx) => Op {
      code: 0x95,
      size: 2,
      cycles: 4,
      check: false,
      mask: 0b00000000,
    },
    ("sta", Abs) => Op {
      code: 0x8D,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b00000000,
    },
    ("sta", Abx) => Op {
      code: 0x9D,
      size: 3,
      cycles: 5,
      check: false,
      mask: 0b00000000,
    },
    ("sta", Aby) => Op {
      code: 0x99,
      size: 3,
      cycles: 5,
      check: false,
      mask: 0b00000000,
    },
    ("sta", Izx) => Op {
      code: 0x81,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b00000000,
    },
    ("sta", Izy) => Op {
      code: 0x91,
      size: 2,
      cycles: 6,
      check: false,
      mask: 0b00000000,
    },
    ("stx", Zpo) => Op {
      code: 0x86,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b00000000,
    },
    ("stx", Zpy) => Op {
      code: 0x96,
      size: 2,
      cycles: 4,
      check: false,
      mask: 0b00000000,
    },
    ("stx", Abs) => Op {
      code: 0x8E,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b00000000,
    },
    ("sty", Zpo) => Op {
      code: 0x84,
      size: 2,
      cycles: 3,
      check: false,
      mask: 0b00000000,
    },
    ("sty", Zpx) => Op {
      code: 0x94,
      size: 2,
      cycles: 4,
      check: false,
      mask: 0b00000000,
    },
    ("sty", Abs) => Op {
      code: 0x8C,
      size: 3,
      cycles: 4,
      check: false,
      mask: 0b00000000,
    },
    ("tax", Imp) => Op {
      code: 0xAA,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("tay", Imp) => Op {
      code: 0xA8,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("tsx", Imp) => Op {
      code: 0xBA,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("txa", Imp) => Op {
      code: 0x8A,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    ("txs", Imp) => Op {
      code: 0x9A,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b00000000,
    },
    ("tya", Imp) => Op {
      code: 0x98,
      size: 1,
      cycles: 2,
      check: false,
      mask: 0b10000010,
    },
    (_, _) => panic!("invalid instruction"),
  }
}

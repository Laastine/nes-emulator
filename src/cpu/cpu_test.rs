use std::cell::RefCell;
use std::rc::Rc;
use crate::apu::Apu;
use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::cpu::instruction_table::{AddrMode6502, Instruction6502};
use crate::cpu::instruction_table::AddrMode6502::*;
use crate::nes::{OffScreenBuffer, controller::Controller};
use crate::ppu::registers::Registers;
use crate::nes::constants::{SCREEN_RES_X, SCREEN_RES_Y};

fn init_cpu() -> Cpu {
  let cartridge = Cartridge::mock_cartridge();
  let cart = Rc::new(RefCell::new(Box::new(cartridge)));

  let controller = Rc::new(RefCell::new(Controller::new()));

  let apu = Rc::new(RefCell::new(Apu::new()));
  let registers = Rc::new(RefCell::new(Registers::new(cart.clone())));

  let off_screen: OffScreenBuffer = [[0u8; 3]; (SCREEN_RES_X * SCREEN_RES_Y) as usize];
  let off_screen_pixels = Rc::new(RefCell::new(off_screen));
  let ppu = Rc::new(RefCell::new(Ppu::new(registers, off_screen_pixels.clone())));

  let bus = Bus::new(cart, controller.clone(), ppu.clone(), apu.clone());

  let cpu = Cpu::new(Rc::new(RefCell::new(bus)));

  cpu
}

macro_rules! build_cpu_and_memory {
    ($bytes: expr) => {
      {
        let mut cpu = init_cpu();

        let bytes = $bytes;
        for (idx, &b) in bytes.iter().enumerate() {
          cpu.get_mut_bus().ram[idx] = b as u8;
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

        let start_p = cpu.status_register;
        $(cpu.$sk=$sv;)*
        cpu.tick();
        assert!(0 == cpu.status_register & start_p & !op.mask, "Register mask not respected. P: 0b{:b}", cpu.status_register);

        if op.size > 0 {
            assert!(op.size == (cpu.pc - start_pc), "Invalid instruction size. Expected: {} bytes, Got: {}", op.size, cpu.pc - start_pc);
        }

        if op.cycles > 0 {
          assert!(op.cycles == (cpu.cycle - start_cycles), "Invalid instruction duration. Expected: {} cycles, Got: {}", op.cycles, cpu.cycle - start_cycles);
        }

        $(
            assert!(cpu.$ek==$ev, "Incorrect Register. Expected cpu.{} to be {}, got {:#010b}", stringify!($ek), stringify!($ev), cpu.$ek);
        )*
        let mut mem = Vec::new();
        $(mem.push($rb);)*
        mem.insert(0, op.code);
        for (i, &b) in mem.iter().enumerate() {
            assert!(cpu.get_mut_bus().ram[i]==b, "Incorrect Memory. Expected ram[{}] to be {}, got 0x{:04X}", i, b, cpu.get_mut_bus().ram[i]);
        }

        cpu
      }
    }
}

#[test]
fn test_lda() {
  test_op_code!("lda", Imm, [0x00]{} => []{ acc: 0x00, status_register: 0b00000010 });
  test_op_code!("lda", Imm, [0xFF]{} => []{ acc: 0xFF, status_register: 0b10000000 });
  test_op_code!("lda", Imm, [0x20]{} => []{ acc: 0x20, status_register: 0 });
  test_op_code!("lda", Zpo,  [0x02, 0x90]{} => []{ acc: 0x90 });
  test_op_code!("lda", Zpx, [0x02, 0, 0x90]{x:1} => []{ acc: 0x90 });
  test_op_code!("lda", Abs,  [0x04, 0, 0, 0x90]{x:1} => []{ acc: 0x90 });
  test_op_code!("lda", Abx, [0x03, 0, 0, 0x90]{x:1} => []{ acc: 0x90 });
  test_op_code!("lda", Aby, [0x03, 0, 0, 0x90]{y:1} => []{ acc: 0x90 });
  test_op_code!("lda", Izx, [0x02, 0, 0x05, 0, 0x90]{x:1} => []{ acc: 0x90 });
  test_op_code!("lda", Izy, [0x02, 0x04, 0, 0, 0x90]{y:1} => []{ acc: 0x90 });
}

#[test]
fn test_ldx() {
  test_op_code!("ldx", Imm, [0x00]{}                 => []{ x: 0x00, status_register: 0b00000010 });
  test_op_code!("ldx", Imm, [0xFF]{}                 => []{ x: 0xFF, status_register: 0b10000000 });
  test_op_code!("ldx", Imm, [0x20]{}                 => []{ x: 0x20, status_register: 0 });
  test_op_code!("ldx", Zpo,  [0x02, 0x90]{}           => []{ x: 0x90 });
  test_op_code!("ldx", Zpy, [0x02, 0, 0x90]{y:1}     => []{ x: 0x90 });
  test_op_code!("ldx", Abs,  [0x04, 0, 0, 0x90]{}     => []{ x: 0x90 });
  test_op_code!("ldx", Aby, [0x03, 0, 0, 0x90]{y:1}  => []{ x: 0x90 });
}

#[test]
fn test_ldy() {
  test_op_code!("ldy", Imm, [0x00]{} => []{ y: 0x00, status_register: 0b00000010 });
  test_op_code!("ldy", Imm, [0xFF]{} => []{ y: 0xFF, status_register: 0b10000000 });
  test_op_code!("ldy", Imm, [0x20]{} => []{ y: 0x20, status_register: 0 });
  test_op_code!("ldy", Zpo,  [0x02, 0x90]{} => []{ y: 0x90 });
  test_op_code!("ldy", Zpx, [0x02, 0, 0x90]{x:1} => []{ y: 0x90 });
  test_op_code!("ldy", Abs,  [0x04, 0, 0, 0x90]{x:1} => []{ y: 0x90 });
  test_op_code!("ldy", Abx, [0x03, 0, 0, 0x90]{x:1} => []{ y: 0x90 });
}

#[test]
fn test_sta() {
  test_op_code!("sta", Zpo,  [0x02]{acc: 0x66} => [0x02, 0x66]{});
  test_op_code!("sta", Zpx, [0x02]{acc: 0x66, x:1} => [0x02, 0, 0x66]{});
  test_op_code!("sta", Abs,  [0x04, 0]{acc: 0x66} => [0x04, 0, 0, 0x66]{});
  test_op_code!("sta", Abx, [0x03, 0]{acc: 0x66, x:1} => [0x03, 0, 0, 0x66]{});
  test_op_code!("sta", Aby, [0x03, 0]{acc: 0x66, y:1} => [0x03, 0, 0, 0x66]{});
  test_op_code!("sta", Izx, [0x02, 0, 0x05, 0, 0]{acc: 0x66, x:1} => [0x02, 0, 0x05, 0, 0x66]{});
  test_op_code!("sta", Izy, [0x02, 0x04, 0, 0, 0]{acc: 0x66, y:1} => [0x02, 0x04, 0, 0, 0x66]{});
}

#[test]
fn test_stx() {
  test_op_code!("stx", Zpo,  [0x02]{x: 0x66} => [0x02, 0x66]{});
  test_op_code!("stx", Zpy, [0x02]{x: 0x66, y:1} => [0x02, 0, 0x66]{});
  test_op_code!("stx", Abs,  [0x04, 0]{x: 0x66} => [0x04, 0, 0, 0x66]{});
}

#[test]
fn test_sty() {
  test_op_code!("sty", Zpo,  [0x02]{y: 0x66} => [0x02, 0x66]{});
  test_op_code!("sty", Zpx, [0x02]{y: 0x66, x:1} => [0x02, 0, 0x66]{});
  test_op_code!("sty", Abs,  [0x04, 0]{y: 0x66} => [0x04, 0, 0, 0x66]{});
}

#[test]
fn test_adc() {
  test_op_code!("adc", Imm, [3]{acc:2, status_register:1} => []{ acc: 6 });
  test_op_code!("adc", Imm, [255]{acc:1, status_register:0} => []{ acc: 0, status_register: 0b00000011 });
  test_op_code!("adc", Imm, [127]{acc:1, status_register:0} => []{ acc: 128, status_register: 0b11000000 });
  test_op_code!("adc", Imm, [200]{acc:100} => []{ acc: 44 });
  test_op_code!("adc", Zpo,  [0x02, 0x90]{acc: 1} => []{ acc: 0x91 });
  test_op_code!("adc", Zpx, [0x02, 0, 0x90]{x:1, acc: 1} => []{ acc: 0x91 });
  test_op_code!("adc", Abs,  [0x04, 0, 0, 0x90]{acc:1} => []{ acc: 0x91 });
  test_op_code!("adc", Abx, [0x03, 0, 0, 0x90]{x:1, acc: 1} => []{ acc: 0x91 });
  test_op_code!("adc", Aby, [0x03, 0, 0, 0x90]{y:1, acc: 1} => []{ acc: 0x91 });
  test_op_code!("adc", Izx, [0x02, 0, 0x05, 0, 0x90]{x:1, acc: 1} => []{ acc: 0x91 });
  test_op_code!("adc", Izy, [0x02, 0x04, 0, 0, 0x90]{y:1, acc: 1} => []{ acc: 0x91 });
}

#[test]
fn test_sbc() {
  test_op_code!("sbc", Imm, [2]{acc:10, status_register:1} => []{ acc: 8 });
  test_op_code!("sbc", Imm, [2]{acc:10, status_register:0} => []{ acc: 7 });
  test_op_code!("sbc", Imm, [176]{acc:80, status_register:1} => []{ acc: 160, status_register: 0b11000000 });
  test_op_code!("sbc", Zpo,  [0x02, 0x90]{acc: 0xFF, status_register: 1} => []{ acc: 0x6f });
  test_op_code!("sbc", Zpx, [0x02, 0, 0x90]{x:1, acc: 0xFF, status_register: 1} => []{ acc: 0x6f });
  test_op_code!("sbc", Abs,  [0x04, 0, 0, 0x90]{acc:0xFF, status_register: 1} => []{ acc: 0x6f });
  test_op_code!("sbc", Abx, [0x03, 0, 0, 0x90]{x:1, acc: 0xFF, status_register: 1} => []{ acc: 0x6f });
  test_op_code!("sbc", Aby, [0x03, 0, 0, 0x90]{y:1, acc: 0xFF, status_register: 1} => []{ acc: 0x6f });
  test_op_code!("sbc", Izx, [0x02, 0, 0x05, 0, 0x90]{x:1, acc: 0xFF, status_register: 1} => []{ acc: 0x6f });
  test_op_code!("sbc", Izy, [0x02, 0x04, 0, 0, 0x90]{y:1, acc: 0xFF, status_register: 1} => []{ acc: 0x6f });
}

#[test]
fn test_cmp() {
  test_op_code!("cmp", Imm, [10]{acc:10} => []{ status_register: 0b00000011 });
  test_op_code!("cmp", Imm, [100]{acc:10} => []{ status_register: 0b10000000 });
  test_op_code!("cmp", Imm, [10]{acc:100} => []{ status_register: 0b00000001 });
  test_op_code!("cmp", Zpo,  [0x02, 10]{acc: 10} => []{ status_register: 0b00000011 });
  test_op_code!("cmp", Zpx, [0x02, 0, 10]{x:1, acc: 10} => []{ status_register: 0b00000011 });
  test_op_code!("cmp", Abs,  [0x04, 0, 0, 10]{acc:10} => []{ status_register: 0b00000011  });
  test_op_code!("cmp", Abx, [0x03, 0, 0, 10]{x:1, acc: 10} => []{ status_register: 0b00000011 });
  test_op_code!("cmp", Aby, [0x03, 0, 0, 10]{y:1, acc: 10} => []{ status_register: 0b00000011 });
  test_op_code!("cmp", Izx, [0x02, 0, 0x05, 0, 10]{x:1, acc: 10} => []{ status_register: 0b00000011 });
  test_op_code!("cmp", Izy, [0x02, 0x04, 0, 0, 10]{y:1, acc: 10} => []{ status_register: 0b00000011 });
}

#[test]
fn test_cpx() {
  test_op_code!("cpx", Imm, [10]{x:10} => []{ status_register: 0b00000011 });
  test_op_code!("cpx", Imm, [100]{x:10} => []{ status_register: 0b10000000 });
  test_op_code!("cpx", Imm, [10]{x:100} => []{ status_register: 0b00000001 });
  test_op_code!("cpx", Zpo,  [0x02, 10]{x: 10} => []{ status_register: 0b00000011 });
  test_op_code!("cpx", Abs,  [0x04, 0, 0, 10]{x:10} => []{ status_register: 0b00000011  });
}

#[test]
fn test_cpy() {
  test_op_code!("cpy", Imm, [10]{y:10} => []{ status_register: 0b00000011 });
  test_op_code!("cpy", Imm, [100]{y:10} => []{ status_register: 0b10000000 });
  test_op_code!("cpy", Imm, [10]{y:100} => []{ status_register: 0b00000001 });
  test_op_code!("cpy", Zpo,  [0x02, 10]{y: 10} => []{ status_register: 0b00000011 });
  test_op_code!("cpy", Abs,  [0x04, 0, 0, 10]{y:10} => []{ status_register: 0b00000011  });
}

#[test]
fn test_and() {
  test_op_code!("and", Imm, [0b00001111]{acc:0b01010101} => []{ acc: 0b00000101, status_register: 0 });
  test_op_code!("and", Imm, [0b10001111]{acc:0b11010101} => []{ acc: 0b10000101, status_register: 0b10000000 });
  test_op_code!("and", Imm, [0]{acc:0b11010101} => []{ acc: 0, status_register: 0b00000010 });
  test_op_code!("and", Zpo,  [0x02, 0xFF]{acc: 0xF0} => []{acc: 0xF0});
  test_op_code!("and", Zpx, [0x02, 0, 0xFF]{x:1, acc: 0xF0} => []{acc: 0xF0});
  test_op_code!("and", Abs,  [0x04, 0, 0, 0xFF]{acc:0xF0} => []{acc: 0xF0});
  test_op_code!("and", Abx, [0x03, 0, 0, 0xFF]{x:1, acc: 0xF0} => []{acc: 0xF0});
  test_op_code!("and", Aby, [0x03, 0, 0, 0xFF]{y:1, acc: 0xF0} => []{acc: 0xF0});
  test_op_code!("and", Izx, [0x02, 0, 0x05, 0, 0xFF]{x:1, acc: 0xF0} => []{acc: 0xF0});
  test_op_code!("and", Izy, [0x02, 0x04, 0, 0, 0xFF]{y:1, acc: 0xF0} => []{acc: 0xF0});
}

#[test]
fn test_ora() {
  test_op_code!("ora", Imm, [0b00001111]{acc:0b01010101} => []{ acc: 0b01011111, status_register: 0 });
  test_op_code!("ora", Imm, [0b10001111]{acc:0b01010101} => []{ acc: 0b11011111, status_register: 0b10000000 });
  test_op_code!("ora", Imm, [0]{acc:0} => []{ acc: 0, status_register: 0b00000010 });
  test_op_code!("ora", Zpo,  [0x02, 0xFF]{acc: 0xF0} => []{acc: 0xFF});
  test_op_code!("ora", Zpx, [0x02, 0, 0xFF]{x:1, acc: 0xF0} => []{acc: 0xFF});
  test_op_code!("ora", Abs,  [0x04, 0, 0, 0xFF]{acc:0xF0} => []{acc: 0xFF});
  test_op_code!("ora", Abx, [0x03, 0, 0, 0xFF]{x:1, acc: 0xF0} => []{acc: 0xFF});
  test_op_code!("ora", Aby, [0x03, 0, 0, 0xFF]{y:1, acc: 0xF0} => []{acc: 0xFF});
  test_op_code!("ora", Izx, [0x02, 0, 0x05, 0, 0xFF]{x:1, acc: 0xF0} => []{acc: 0xFF});
  test_op_code!("ora", Izy, [0x02, 0x04, 0, 0, 0xFF]{y:1, acc: 0xF0} => []{acc: 0xFF});
}

#[test]
fn test_eor() {
  test_op_code!("eor", Imm, [0b00001111]{acc:0b01010101} => []{ acc: 0b01011010, status_register: 0 });
  test_op_code!("eor", Imm, [0b10001111]{acc:0b01010101} => []{ acc: 0b11011010, status_register: 0b10000000 });
  test_op_code!("eor", Imm, [0xFF]{acc:0xFF} => []{ acc: 0, status_register: 0b00000010 });
  test_op_code!("eor", Zpo,  [0x02, 0xFF]{acc: 0xF0} => []{acc: 0x0F});
  test_op_code!("eor", Zpx, [0x02, 0, 0xFF]{x:1, acc: 0xF0} => []{acc: 0x0F});
  test_op_code!("eor", Abs,  [0x04, 0, 0, 0xFF]{acc:0xF0} => []{acc: 0x0F});
  test_op_code!("eor", Abx, [0x03, 0, 0, 0xFF]{x:1, acc: 0xF0} => []{acc: 0x0F});
  test_op_code!("eor", Aby, [0x03, 0, 0, 0xFF]{y:1, acc: 0xF0} => []{acc: 0x0F});
  test_op_code!("eor", Izx, [0x02, 0, 0x05, 0, 0xFF]{x:1, acc: 0xF0} => []{acc: 0x0F});
  test_op_code!("eor", Izy, [0x02, 0x04, 0, 0, 0xFF]{y:1, acc: 0xF0} => []{acc: 0x0F});
}

#[test]
fn test_bit() {
  test_op_code!("bit", Zpo,  [0x02, 0x00]{acc: 0x0F} => []{status_register: 0b00000010});
  test_op_code!("bit", Zpo,  [0x02, 0xF0]{acc: 0xFF} => []{status_register: 0b11000000});
  test_op_code!("bit", Abs,  [0x03, 0, 0xF0]{acc: 0xFF} => []{status_register: 0b11000000});
}

#[test]
fn test_rol() {
  test_op_code!("rol", Zpo,  [0x02, 0xFF]{status_register:1} => [0x02, 0xFF]{status_register: 0b10000001});
  test_op_code!("rol", Zpo,  [0x02, 0xFF]{status_register:0} => [0x02, 0xFE]{status_register: 0b10000001});
  test_op_code!("rol", Zpo,  [0x02, 0b10000000]{status_register:0} => [0x02, 0]{status_register: 0b00000011});
  test_op_code!("rol", Zpx, [0x02, 0, 0xFF]{status_register:1, x: 1} => [0x02, 0, 0xFF]{status_register: 0b10000001});
  test_op_code!("rol", Abs,  [0x03, 0, 0xFF]{status_register:1} => [0x03, 0, 0xFF]{status_register: 0b10000001});
  test_op_code!("rol", Abx, [0x03, 0, 0, 0xFF]{status_register:1, x: 1} => [0x03, 0, 0, 0xFF]{status_register: 0b10000001});
}

#[test]
fn test_ror() {
  test_op_code!("ror", Zpo,  [0x02, 0xFF]{status_register:1} => [0x02, 0xFF]{status_register: 0b10000001});
  test_op_code!("ror", Zpo,  [0x02, 0xFF]{status_register:0} => [0x02, 0x7f]{status_register: 0b00000001});
  test_op_code!("ror", Zpo,  [0x02, 1]{status_register:0} => [0x02, 0]{status_register: 0b00000011});
  test_op_code!("ror", Zpx,  [0x02, 0, 1]{status_register:0, x: 1} => [0x02, 0]{status_register: 0b00000011});
  test_op_code!("ror", Abs,  [0x03, 0, 1]{status_register:0} => [0x03, 0]{status_register: 0b00000011});
  test_op_code!("ror", Abx,  [0x02, 0, 1]{status_register:0, x: 1} => [0x02, 0]{status_register: 0b00000011});
}

#[test]
fn test_asl() {
  test_op_code!("asl", Zpo,  [0x02, 0xFF]{status_register:1} => [0x02, 0xFE]{status_register: 0b10000001});
  test_op_code!("asl", Zpo,  [0x02, 0xFF]{status_register:0} => [0x02, 0xFE]{status_register: 0b10000001});
  test_op_code!("asl", Zpo,  [0x02, 0b10000000]{} => [0x02, 0]{status_register: 0b00000011});
  test_op_code!("asl", Zpx, [0x02, 0, 1]{x: 1} => [0x02, 0, 2]{});
  test_op_code!("asl", Abs,  [0x03, 0, 1]{} => [0x03, 0, 2]{});
  test_op_code!("asl", Abx, [0x03, 0, 0, 1]{x: 1} => [0x03, 0, 0, 2]{});
}

#[test]
fn test_lsr() {
  test_op_code!("lsr", Zpo,  [0x02, 1]{status_register:1} => [0x02, 0]{status_register: 0b00000011});
  test_op_code!("lsr", Zpo,  [0x02, 1]{status_register:0} => [0x02, 0]{status_register: 0b00000011});
  test_op_code!("lsr", Zpx, [0x02, 0, 2]{x: 1} => [0x02, 0, 1]{});
  test_op_code!("lsr", Abs,  [0x03, 0, 2]{} => [0x03, 0, 1]{});
  test_op_code!("lsr", Abx, [0x03, 0, 0, 2]{x: 1} => [0x03, 0, 0, 1]{});
  test_op_code!("lsr", Imp, []{acc: 2} => []{acc: 1});
}

#[test]
fn test_inc() {
  test_op_code!("inc", Zpo,  [0x02, 255]{} => [0x02, 0]{status_register: 0b00000010});
  test_op_code!("inc", Zpo,  [0x02, 127]{} => [0x02, 128]{status_register: 0b10000000});
  test_op_code!("inc", Zpx, [0x02, 0, 2]{x: 1} => [0x02, 0, 3]{});
  test_op_code!("inc", Abs,  [0x03, 0, 2]{} => [0x03, 0, 3]{});
  test_op_code!("inc", Abx, [0x03, 0, 0, 2]{x: 1} => [0x03, 0, 0, 3]{});
}

#[test]
fn test_dec() {
  test_op_code!("dec", Zpo,  [0x02, 0]{} => [0x02, 255]{status_register: 0b10000000});
  test_op_code!("dec", Zpo,  [0x02, 1]{} => [0x02, 0]{status_register: 0b00000010});
  test_op_code!("dec", Zpx, [0x02, 0, 2]{x: 1} => [0x02, 0, 1]{});
  test_op_code!("dec", Abs,  [0x03, 0, 2]{} => [0x03, 0, 1]{});
  test_op_code!("dec", Abx, [0x03, 0, 0, 2]{x: 1} => [0x03, 0, 0, 1]{});
}

#[test]
fn test_inx() {
  test_op_code!("inx", Imp,  []{x: 255} => []{x: 0, status_register: 0b00000010});
  test_op_code!("inx", Imp,  []{x: 127} => []{x: 128, status_register: 0b10000000});
}

#[test]
fn test_dex() {
  test_op_code!("dex", Imp,  []{x: 1} => []{x: 0, status_register: 0b00000010});
  test_op_code!("dex", Imp,  []{x: 0} => []{x: 255, status_register: 0b10000000});
}

#[test]
fn test_iny() {
  test_op_code!("iny", Imp,  []{y: 255} => []{y: 0, status_register: 0b00000010});
  test_op_code!("iny", Imp,  []{y: 127} => []{y: 128, status_register: 0b10000000});
}

#[test]
fn test_dey() {
  test_op_code!("dey", Imp,  []{y: 1} => []{y: 0, status_register: 0b00000010});
  test_op_code!("dey", Imp,  []{y: 0} => []{y: 255, status_register: 0b10000000});
}

#[test]
fn test_tax() {
  test_op_code!("tax", Imp,  []{acc: 1} => []{acc: 1, x: 1, status_register: 0b00000000});
  test_op_code!("tax", Imp,  []{acc: 0} => []{acc: 0, x: 0, status_register: 0b00000010});
  test_op_code!("tax", Imp,  []{acc: 128} => []{acc: 128, x: 128, status_register: 0b10000000});
}

#[test]
fn test_tay() {
  test_op_code!("tay", Imp,  []{acc: 1} => []{acc: 1, y: 1, status_register: 0b00000000});
  test_op_code!("tay", Imp,  []{acc: 0} => []{acc: 0, y: 0, status_register: 0b00000010});
  test_op_code!("tay", Imp,  []{acc: 128} => []{acc: 128, y: 128, status_register: 0b10000000});
}
#[test]
fn test_txa() {
  test_op_code!("txa", Imp,  []{x: 1} => []{acc: 1, x: 1, status_register: 0b00000000});
  test_op_code!("txa", Imp,  []{x: 0} => []{acc: 0, x: 0, status_register: 0b00000010});
  test_op_code!("txa", Imp,  []{x: 128} => []{acc: 128, x: 128, status_register: 0b10000000});
}

#[test]
fn test_tya() {
  test_op_code!("tya", Imp,  []{y: 1} => []{acc: 1, y: 1, status_register: 0b00000000});
  test_op_code!("tya", Imp,  []{y: 0} => []{acc: 0, y: 0, status_register: 0b00000010});
  test_op_code!("tya", Imp,  []{y: 128} => []{acc: 128, y: 128, status_register: 0b10000000});
}

#[test]
fn test_tsx() {
  test_op_code!("tsx", Imp,  []{stack_pointer: 1} => []{stack_pointer: 0b00000001, x: 1, status_register: 0b00000000});
  test_op_code!("tsx", Imp,  []{stack_pointer: 0} => []{stack_pointer: 0b00000000, x: 0, status_register: 0b00000010});
  test_op_code!("tsx", Imp,  []{stack_pointer: 128} => []{stack_pointer: 0b10000000, x: 128, status_register: 0b10000000});
}

#[test]
fn test_txs() {
  test_op_code!("txs", Imp,  []{x: 1} => []{stack_pointer: 1, x: 1, status_register: 0});
}

#[test]
fn test_flag_ops() {
  test_op_code!("clc", Imp, []{status_register: 0b11111111} => []{status_register: 0b11111110});
  test_op_code!("sec", Imp, []{status_register: 0} => []{status_register: 1});
  test_op_code!("cli", Imp, []{status_register: 0b11111111} => []{status_register: 0b11111011});
  test_op_code!("sei", Imp, []{status_register: 0} => []{status_register: 0b00000100});
  test_op_code!("clv", Imp, []{status_register: 0b11111111} => []{status_register: 0b10111111});
  test_op_code!("cld", Imp, []{status_register: 0b11111111} => []{status_register: 0b11110111});
  test_op_code!("sed", Imp, []{status_register: 0} => []{status_register: 0b00001000});
}

// #[test]
// fn test_bpl() {
//   let cpu = test_op_code!("bpl", Imp, [10]{status_register: 0b10000000} => []{pc: 0b00000010});
//   assert_eq!(cpu.cycle, 2);
//
//   let cpu = test_op_code!("bpl", Imp, [10]{status_register: 0} => []{pc: 12});
//   assert_eq!(cpu.cycle, 3);
//
//   let mut cpu = build_cpu_and_memory!([0]);
//   cpu.pc = 0x00FE;
//   cpu.bus.ram[0x00FE] = 1;
//   cpu.bpl();
//   assert!(cross(0x00FF, 1));
//   assert_eq!(cpu.pc, 0x0100);
//   assert_eq!(cpu.cycle, 3);
// }
//
// #[test]
// fn test_bmi() {
//   let cpu = test_op_code!("bmi", Imp, [10]{status_register: 0} => []{pc: 2});
//   assert_eq!(cpu.cycle, 2);
//
//   let cpu = test_op_code!("bmi", Imp, [10]{status_register: 0b10000000} => []{pc: 12});
//   assert_eq!(cpu.cycle, 3);
//
//   let mut cpu = build_cpu_and_memory!([0]);
//   cpu.pc = 0x00FE;
//   cpu.bus.ram[0x00FE] = 1;
//   cpu.stack_pointer = 0b10000000;
//   cpu.bmi();
//   assert!(cross(0x00FF, 1));
//   assert_eq!(cpu.pc, 0x0100);
//   assert_eq!(cpu.cycle, 3);
// }
//
// fn cross(base: u16, offset: u8) -> bool {
//   high_byte(base + offset as u16) != high_byte(base)
// }
//
// fn high_byte(value: u16) -> u16 {
//   value & 0xFF00
// }

fn high_byte(value: u16) -> u16 {
  value & 0xFF00
}

#[derive(Copy, Clone)]
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

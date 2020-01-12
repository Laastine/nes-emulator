use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{stdout, Stdout, Write};
use std::rc::Rc;
use std::time;

use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::render_state::RenderState;
use luminance_glutin::{ElementState, Event, KeyboardInput, Surface, VirtualKeyCode, WindowEvent};
use termion::{clear, color, cursor, style};
use termion::raw::{IntoRawMode, RawTerminal};

use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::{Cpu, hex};
use crate::cpu::instruction_table::FLAGS6502;
use crate::gfx::WindowContext;
use crate::ppu::{Ppu, registers::Registers};

pub mod constants;

const RED: color::Fg<color::AnsiValue> = color::Fg(color::AnsiValue(196));
const GREEN: color::Fg<color::AnsiValue> = color::Fg(color::AnsiValue(46));

pub struct Nes {
  cpu: Cpu,
  ppu: Ppu,
  map_asm: HashMap<u16, String>,
  system_cycles: u64,
  window_context: WindowContext,
  debug_mode: bool,
}

impl Nes {
  pub fn new(rom_file: &str) -> Nes {
    let cartridge = Cartridge::new(rom_file);
    let cart = Rc::new(RefCell::new(cartridge));

    let map_asm: HashMap<u16, String> = HashMap::new();

    let mut window_context = WindowContext::new();

    let reg = Registers::new(cart.clone());
    let registers = Rc::new(RefCell::new(reg));

    let bus = Bus::new(cart, registers.clone());

    let cpu = Cpu::new(bus);
    let ppu = Ppu::new(registers, &mut window_context.surface);
    let system_cycles = 0;

    let debug_mode = false;

    Nes {
      cpu,
      ppu,
      map_asm,
      system_cycles,
      window_context,
      debug_mode,
    }
  }

  fn draw_ram(
    &mut self,
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
          hex(self.cpu.bus.read_u8(addr, true).try_into().unwrap(), 2)
        );
        addr += 1;
      }
      write!(stdout, "{}{}", cursor::Goto(x_ram, y_ram), offset).unwrap();
      y_ram += 1;
    }
  }

  fn draw_cpu(&self, stdout: &mut RawTerminal<Stdout>, x: u16, y: u16) {
    write!(stdout, "{}Status", cursor::Goto(x, y)).unwrap();
    if self.cpu.status_register & FLAGS6502::N.value() > 0x00 {
      write!(stdout, "{}{}N", cursor::Goto(x + 7, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}N", cursor::Goto(x + 7, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::V.value() > 0x00 {
      write!(stdout, "{}{}V", cursor::Goto(x + 9, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}V", cursor::Goto(x + 9, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::U.value() > 0x00 {
      write!(stdout, "{}{}-", cursor::Goto(x + 11, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}-", cursor::Goto(x + 11, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::B.value() > 0x00 {
      write!(stdout, "{}{}B", cursor::Goto(x + 13, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}B", cursor::Goto(x + 13, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::D.value() > 0x00 {
      write!(stdout, "{}{}D", cursor::Goto(x + 15, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}D", cursor::Goto(x + 15, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::I.value() > 0x00 {
      write!(stdout, "{}{}I", cursor::Goto(x + 17, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}I", cursor::Goto(x + 17, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::Z.value() > 0x00 {
      write!(stdout, "{}{}Z", cursor::Goto(x + 19, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}Z", cursor::Goto(x + 19, y), RED).unwrap();
    }
    if self.cpu.status_register & FLAGS6502::C.value() > 0x00 {
      write!(stdout, "{}{}C", cursor::Goto(x + 21, y), GREEN).unwrap();
    } else {
      write!(stdout, "{}{}C", cursor::Goto(x + 21, y), RED).unwrap();
    }
    writeln!(stdout, "{}", style::Reset).unwrap();

    write!(
      stdout,
      "{}PC: ${}",
      cursor::Goto(x, y + 1),
      hex(self.cpu.pc.try_into().unwrap(), 4)
    )
    .unwrap();
    write!(
      stdout,
      "{}A: ${} [{}]",
      cursor::Goto(x, y + 2),
      hex(self.cpu.acc.try_into().unwrap(), 2),
      self.cpu.acc
    )
    .unwrap();
    write!(
      stdout,
      "{}X: ${} [{}]",
      cursor::Goto(x, y + 3),
      hex(self.cpu.x.try_into().unwrap(), 2),
      self.cpu.x
    )
    .unwrap();
    write!(
      stdout,
      "{}Y: ${} [{}]",
      cursor::Goto(x, y + 4),
      hex(self.cpu.y.try_into().unwrap(), 2),
      self.cpu.y
    )
    .unwrap();

    write!(
      stdout,
      "{}Stack P: ${}",
      cursor::Goto(x, y + 5),
      hex(usize::from(self.cpu.stack_pointer), 4)
    )
    .unwrap();
  }

  pub fn draw_code(&self, stdout: &mut RawTerminal<Stdout>, x: u16, y: u16) {
    let val = self.map_asm.get(&self.cpu.pc);
    if val.is_some() {
      write!(stdout, "{}{}", cursor::Goto(x, y), clear::AfterCursor).unwrap();
      write!(
        stdout,
        "{}{}{}",
        cursor::Goto(x, y),
        val.unwrap(),
        style::Reset
      )
        .unwrap();
    }
  }

  pub fn create_program(&mut self) {
    self.map_asm = self.cpu.disassemble(0x0000, 0xFFFF);
    self.reset();
  }

  fn draw_terminal(&mut self, stdout: &mut RawTerminal<Stdout>) {
    self.draw_ram(stdout, 0x0000, 2, 2, 16, 16);
    self.draw_ram(stdout, 0x8000, 2, 20, 16, 16);
    self.draw_cpu(stdout, 64, 2);
    self.draw_code(stdout, 64, 9);
  }

  pub fn render_loop(&mut self) {
    let mut stdout = stdout()
      .into_raw_mode()
      .unwrap_or_else(|err| panic!("stdout raw mode error {:?}", err));

    write!(stdout, "{}{}", cursor::Goto(1, 1), clear::AfterCursor).unwrap();

    let mut last_time = time::Instant::now();

    'app: loop {
      let elapsed = last_time.elapsed();
      let delta = f64::from(elapsed.subsec_nanos()) / 1e9 + elapsed.as_secs() as f64;

      for event in self.window_context.surface.poll_events() {
        if let Event::WindowEvent { event, .. } = event {
          match event {
            WindowEvent::CloseRequested
            | WindowEvent::Destroyed
            | WindowEvent::KeyboardInput {
              input:
                KeyboardInput {
                  state: ElementState::Released,
                  virtual_keycode: Some(VirtualKeyCode::Escape),
                  ..
                },
              ..
            } => {
              write!(stdout, "{}{}", cursor::Goto(1, 1), clear::AfterCursor).unwrap();
              break 'app;
            }
            WindowEvent::KeyboardInput {
              input:
                KeyboardInput {
                  state: ElementState::Pressed,
                  virtual_keycode: Some(VirtualKeyCode::Space),
                  ..
                },
              ..
            } => {
              self.debug_mode = !self.debug_mode;
            }
            WindowEvent::KeyboardInput {
              input:
                KeyboardInput {
                  state: ElementState::Released,
                  virtual_keycode: Some(VirtualKeyCode::R),
                  ..
                },
              ..
            } => {
              self.cpu.reset();
            }
            WindowEvent::Resized(_) | WindowEvent::HiDpiFactorChanged(_) => {
              self.window_context.resize = true;
            }
            _ => (),
          }
        }
      }

      if !self.debug_mode {
        self.clock();
      }

      if delta > 0.033 {
        last_time = time::Instant::now();
//        self.draw_terminal(&mut stdout);
        if self.ppu.frame_ready {
          self.render_screen();
          self.ppu.frame_ready = false;
        }
      }
    }
  }

  fn clock(&mut self) {
    self.ppu.clock();
    if (self.system_cycles % 3) == 0 {
      self.cpu.clock();
    }

    if self.ppu.nmi {
      self.ppu.nmi = false;
      self.cpu.nmi();
    }

    self.system_cycles = self.system_cycles.wrapping_add(1);
  }

  fn render_screen(&mut self) {
    if self.window_context.resize {
      self.window_context.back_buffer = self.window_context.surface.back_buffer().unwrap();
      let size = self.window_context.surface.size();
      self.window_context.front_buffer =
        Framebuffer::new(&mut self.window_context.surface, size, 0)
          .expect("Framebuffer recreate error");
      self.window_context.resize = false;
    }

    let mut builder = self.window_context.surface.pipeline_builder();
    let texture = &self.ppu.texture;
    let program = &self.window_context.program;
    let copy_program = &self.window_context.copy_program;
    let triangle = &self.window_context.triangle;
    let background = &self.window_context.background;

    builder.pipeline(
      &self.window_context.front_buffer,
      [0.0, 0.0, 0.0, 0.0],
      |_, mut shd_gate| {
        shd_gate.shade(program, |_, mut rdr_gate| {
          rdr_gate.render(RenderState::default(), |mut tess_gate| {
            tess_gate.render(triangle)
          });
        });
      },
    );

    builder.pipeline(
      &self.window_context.back_buffer,
      [0.0, 0.0, 0.0, 0.0],
      |pipeline, mut shd_gate| {
        let bound_texture = pipeline.bind_texture(texture);

        shd_gate.shade(copy_program, |iface, mut rdr_gate| {
          iface.texture.update(&bound_texture);
          rdr_gate.render(RenderState::default(), |mut tess_gate| {
            tess_gate.render(background)
          });
        });
      },
    );

    self.window_context.surface.swap_buffers();
  }

  fn reset(&mut self) {
    self.cpu.reset();
    self.ppu.reset();
    self.system_cycles = 0;
  }
}

use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{stdout, Stdout, Write};
use std::rc::Rc;
use std::time;

use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::render_state::RenderState;
use luminance_glutin::{ElementState, ElementState::Pressed, Event, KeyboardInput, Surface, VirtualKeyCode::{A, Down, Escape, Left, R, Right, S, Space, Up, X, Z}, WindowEvent};

use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::cpu::instruction_table::FLAGS6502;
use crate::gfx::WindowContext;
use crate::ppu::{Ppu, registers::Registers};
use termion::{cursor, clear};
use termion::raw::IntoRawMode;

pub mod constants;

pub struct Nes {
  cpu: Cpu,
  ppu: Ppu,
  system_cycles: u64,
  window_context: WindowContext,
  debug_mode: bool,
  controller: [u8; 2],
}

impl Nes {
  pub fn new(rom_file: &str) -> Nes {
    let cartridge = Cartridge::new(rom_file);
    let cart = Rc::new(RefCell::new(cartridge));
    let controller = [0u8; 2];

    let mut window_context = WindowContext::new();

    let reg = Registers::new(cart.clone());
    let registers = Rc::new(RefCell::new(reg));

    let c = Rc::new(RefCell::new(controller));

    let bus = Bus::new(cart, registers.clone(), c);

    let cpu = Cpu::new(bus);
    let ppu = Ppu::new(registers, &mut window_context.surface);
    let system_cycles = 0;

    let debug_mode = false;

    Nes {
      cpu,
      ppu,
      system_cycles,
      window_context,
      debug_mode,
      controller,
    }
  }

  pub fn init(&mut self) {
    self.reset();
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

      self.controller[0] = 0x00;
      for event in self.window_context.surface.poll_events() {
        if let Event::WindowEvent { event, .. } = event {
          match event {
            WindowEvent::CloseRequested
            | WindowEvent::Destroyed
            | WindowEvent::KeyboardInput {
              input:
              KeyboardInput {
                state: ElementState::Released,
                virtual_keycode: Some(Escape),
                ..
              },
              ..
            } => {
              write!(stdout, "{}{}", cursor::Goto(1, 1), clear::AfterCursor).unwrap();
              break 'app;
            }
            WindowEvent::KeyboardInput { input, .. } => {
              match input {
                KeyboardInput { state, virtual_keycode: Some(Z), .. } => {
                  self.controller[0] |= if state == Pressed { 0x40 } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(A), .. } => {
                  self.controller[0] |= if state == Pressed { 0x20 } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(S), .. } => {
                  self.controller[0] |= if state == Pressed { 0x10 } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(X), .. } => {
                  self.controller[0] |= if state == Pressed { 0x80 } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(Up), .. } => {
                  self.controller[0] |= if state == Pressed { 0x08 } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(Down), .. } => {
                  self.controller[0] |= if state == Pressed { 0x04 } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(Left), .. } => {
                  self.controller[0] |= if state == Pressed { 0x02 } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(Right), .. } => {
                  self.controller[0] |= if state == Pressed { 0x01 } else { 0 };
                }
                KeyboardInput { virtual_keycode: Some(Space), .. } => {
                  self.debug_mode = !self.debug_mode;
                }
                KeyboardInput { state: Pressed, virtual_keycode: Some(R), .. } => {
                  self.cpu.reset();
                }
                _ => {}
              }
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

      if delta > 0.016 {
        last_time = time::Instant::now();
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

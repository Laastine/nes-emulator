use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::time;

use glutin::{ElementState, ElementState::Pressed, KeyboardInput, VirtualKeyCode::{A, Down, Escape, LAlt, LControl, Left, R, Right, S, Up}, WindowEvent};
use luminance::context::GraphicsContext as _;
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::texture::Sampler;

use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::gfx::WindowContext;
use crate::nes::constants::{KeyboardCommand, KeyCodes};
use crate::ppu::{Ppu, registers::Registers};

pub mod constants;

pub struct Nes {
  cpu: Cpu,
  ppu: Ppu,
  system_cycles: u32,
  window_context: WindowContext,
  controller: Rc<RefCell<[u8; 2]>>,
}

impl Nes {
  pub fn new(rom_file: &str) -> Nes {
    let cartridge = Cartridge::new(rom_file);
    let cart = Rc::new(RefCell::new(cartridge));
    let c = [0u8; 2];

    let mut window_context = WindowContext::new();

    let reg = Registers::new(cart.clone());
    let registers = Rc::new(RefCell::new(reg));

    let controller = Rc::new(RefCell::new(c));

    let bus = Bus::new(cart, registers.clone(), controller.clone());

    let cpu = Cpu::new(bus);
    let ppu = Ppu::new(registers, &mut window_context.surface);
    let system_cycles = 0;

    Nes {
      cpu,
      ppu,
      system_cycles,
      window_context,
      controller,
    }
  }

  pub fn init(&mut self) {
    self.reset();
  }

  #[inline]
  fn get_controller(&mut self) -> RefMut<[u8; 2]> {
    self.controller.borrow_mut()
  }

  pub fn render_loop(&mut self) {
    let mut last_time = time::Instant::now();

    let mut keyboard_state = None;
    let mut controller_button_state = 0x00;
    'app: loop {
      let elapsed = last_time.elapsed();
      let delta = elapsed.subsec_millis();

      self.window_context.surface.event_loop.poll_events(|event| {
        if let glutin::Event::WindowEvent { event, .. } = event {
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
            } => keyboard_state = Some(KeyboardCommand::Exit),
            WindowEvent::KeyboardInput { input, .. } => {
              match input {
                KeyboardInput { state, virtual_keycode: Some(LAlt), .. } => {
                  controller_button_state = if state == Pressed { KeyCodes::ButtonA.value() } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(LControl), .. } => {
                  controller_button_state = if state == Pressed { KeyCodes::ButtonB.value() } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(A), .. } => {
                  controller_button_state = if state == Pressed { KeyCodes::Select.value() } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(S), .. } => {
                  controller_button_state = if state == Pressed { KeyCodes::Start.value() } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(Up), .. } => {
                  controller_button_state = if state == Pressed { KeyCodes::Up.value() } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(Down), .. } => {
                  controller_button_state = if state == Pressed { KeyCodes::Down.value() } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(Left), .. } => {
                  controller_button_state = if state == Pressed { KeyCodes::Left.value() } else { 0 };
                }
                KeyboardInput { state, virtual_keycode: Some(Right), .. } => {
                  controller_button_state = if state == Pressed { KeyCodes::Right.value() } else { 0 };
                }
                KeyboardInput { state: Pressed, virtual_keycode: Some(R), .. } => {
                  keyboard_state = Some(KeyboardCommand::Reset)
                }
                _ => {}
              }
            }
            WindowEvent::Resized(_) => {
              keyboard_state = Some(KeyboardCommand::Resize)
            }
            _ => (),
          }
        };
      });

      match keyboard_state {
        Some(KeyboardCommand::Exit) => break 'app,
        Some(KeyboardCommand::Reset) => self.cpu.reset(),
        Some(KeyboardCommand::Resize) => self.window_context.resize = true,
        _ => {}
      }

      if controller_button_state > 0 {
        self.get_controller()[0] |= controller_button_state;
      } else {
        self.get_controller()[0] = 0;
      }

      self.clock();

      // 16ms per frame ~ 60FPS
      if delta > 16 {
        last_time = time::Instant::now();
        if self.ppu.is_frame_ready {
          self.render_screen();
          self.ppu.is_frame_ready = false;
        }
      }
    } // app loop
  }

  fn clock(&mut self) {
    self.ppu.clock();
    if self.cpu.bus.dma_transfer {
      self.cpu.bus.oam_dma_access(self.system_cycles);
    } else if (self.system_cycles % 3) == 0 {
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
        Framebuffer::new(&mut self.window_context.surface, size, 0, Sampler::default())
          .expect("Framebuffer recreate error");
      self.window_context.resize = false;
    }

    let mut builder = self.window_context.surface.pipeline_builder();
    let texture = &self.ppu.texture;
    let program = &self.window_context.program;
    let copy_program = &self.window_context.copy_program;
    let triangle = &self.window_context.texture_vertices;
    let background = &self.window_context.background;

    builder.pipeline(
      &self.window_context.front_buffer,
      &PipelineState::default(),
      |_, mut shd_gate| {
        shd_gate.shade(program, |_, mut rdr_gate| {
          rdr_gate.render(&RenderState::default(), |mut tess_gate| {
            tess_gate.render(triangle)
          });
        });
      },
    );

    builder.pipeline(
      &self.window_context.back_buffer,
      &PipelineState::default(),
      |pipeline, mut shd_gate| {
        let bound_texture = pipeline.bind_texture(texture);

        shd_gate.shade(copy_program, |iface, mut rdr_gate| {
          iface.texture.update(&bound_texture);
          rdr_gate.render(&RenderState::default(), |mut tess_gate| {
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

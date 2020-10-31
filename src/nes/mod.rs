use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;
use std::time;
use std::time::Duration;

use glutin::{ElementState::{Pressed, Released}, KeyboardInput, VirtualKeyCode::{A, Down, Escape, Left, R, Right, S, Up, X, Z}, WindowEvent};
use luminance::context::GraphicsContext as _;
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::texture::Sampler;

use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::gfx::WindowContext;
use crate::nes::constants::{KeyboardCommand, KeyCode};
use crate::ppu::{Ppu, registers::Registers};
use std::borrow::Borrow;

pub mod constants;

pub struct Nes {
  cpu: Cpu,
  ppu: Ppu,
  system_cycles: u32,
  window_context: WindowContext,
  controller: Rc<RefCell<[u8; 2]>>,
}

impl Nes  {
  pub fn new(rom_file: &str) -> Self {
    let rom_bytes = fs::read(rom_file).expect("Rom file read error");

    let cartridge = Cartridge::new(rom_bytes);
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
    fn get_controller_state(key_map: &HashMap<KeyCode, u8>) -> u8 {
      key_map.get(&KeyCode::ButtonB).unwrap_or(&0u8)
        | key_map.get(&KeyCode::ButtonA).unwrap_or(&0u8)
        | key_map.get(&KeyCode::Select).unwrap_or(&0u8)
        | key_map.get(&KeyCode::Start).unwrap_or(&0u8)
        | key_map.get(&KeyCode::Up).unwrap_or(&0u8)
        | key_map.get(&KeyCode::Down).unwrap_or(&0u8)
        | key_map.get(&KeyCode::Left).unwrap_or(&0u8)
        | key_map.get(&KeyCode::Right).unwrap_or(&0u8)
    }

    let mut last_time = time::Instant::now();

    let mut keyboard_state = None;
    let mut key_map: HashMap<KeyCode, u8> = HashMap::new();

    'app: loop {
      let elapsed = last_time.elapsed();
      let delta = elapsed.as_secs_f32();

      // 16ms per frame ~ 60FPS
      if delta > 0.0166 {
        self.window_context.surface.event_loop.poll_events(|event| {
          if let glutin::Event::WindowEvent { event, .. } = event {
            match event {
              WindowEvent::CloseRequested | WindowEvent::Destroyed => { keyboard_state = Some(KeyboardCommand::Exit) }
              WindowEvent::KeyboardInput { input, .. } => {
                match input {
                  KeyboardInput { state: Released, virtual_keycode: Some(Escape), .. } => {
                    keyboard_state = Some(KeyboardCommand::Exit);
                  }
                  KeyboardInput { state, virtual_keycode: Some(Z), .. } => {
                    *key_map.entry(KeyCode::ButtonB)
                      .or_insert_with(|| KeyCode::ButtonB.value()) = if state == Pressed { KeyCode::ButtonB.value() } else { 0 };
                  }
                  KeyboardInput { state, virtual_keycode: Some(X), .. } => {
                    *key_map.entry(KeyCode::ButtonA)
                      .or_insert_with(|| KeyCode::ButtonA.value()) = if state == Pressed { KeyCode::ButtonA.value() } else { 0 };
                  }
                  KeyboardInput { state, virtual_keycode: Some(A), .. } => {
                    *key_map.entry(KeyCode::Select)
                      .or_insert_with(|| KeyCode::Select.value()) = if state == Pressed { KeyCode::Select.value() } else { 0 };
                  }
                  KeyboardInput { state, virtual_keycode: Some(S), .. } => {
                    *key_map.entry(KeyCode::Start)
                      .or_insert_with(|| KeyCode::Start.value()) = if state == Pressed { KeyCode::Start.value() } else { 0 };
                  }
                  KeyboardInput { state, virtual_keycode: Some(Up), .. } => {
                    *key_map.entry(KeyCode::Up)
                      .or_insert_with(|| KeyCode::Up.value()) = if state == Pressed { KeyCode::Up.value() } else { 0 };
                  }
                  KeyboardInput { state, virtual_keycode: Some(Down), .. } => {
                    *key_map.entry(KeyCode::Down)
                      .or_insert_with(|| KeyCode::Down.value()) = if state == Pressed { KeyCode::Down.value() } else { 0 };
                  }
                  KeyboardInput { state, virtual_keycode: Some(Left), .. } => {
                    *key_map.entry(KeyCode::Left)
                      .or_insert_with(|| KeyCode::Left.value()) = if state == Pressed { KeyCode::Left.value() } else { 0 };
                  }
                  KeyboardInput { state, virtual_keycode: Some(Right), .. } => {
                    *key_map.entry(KeyCode::Right)
                      .or_insert_with(|| KeyCode::Right.value()) = if state == Pressed { KeyCode::Right.value() } else { 0 };
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
            };
          }
        });

        match keyboard_state {
          Some(KeyboardCommand::Exit) => break 'app,
          Some(KeyboardCommand::Reset) => self.cpu.reset(),
          Some(KeyboardCommand::Resize) => self.window_context.resize = true,
          _ => {}
        }

        last_time = time::Instant::now();
        if self.ppu.is_frame_ready {
          if keyboard_state == Some(KeyboardCommand::Resize) {
            self.window_context.resize = true;
          }
          self.render_screen();
          self.ppu.is_frame_ready = false;
        }
      } else if delta >= 0.001 {
        self.get_controller()[0] = get_controller_state(&key_map);

        if keyboard_state == Some(KeyboardCommand::Reset) {
          self.cpu.reset();
        }
        self.clock();
      } else {
        std::thread::sleep(Duration::from_millis(1));
      }
    } // app loop
  }

  fn clock(&mut self) {
    self.ppu.clock();

    if (self.system_cycles % 3) == 0 {
      if !self.cpu.bus.dma_transfer {
        self.cpu.clock();
      } else if self.cpu.bus.dma_transfer {
        self.system_cycles = self.system_cycles.wrapping_add(self.cpu.bus.oam_dma_access(self.system_cycles));
      }
    }

    if self.ppu.nmi {
      self.ppu.nmi = false;
      self.cpu.nmi();
    }

    if self.cpu.bus.borrow().get_cartridge().irq_flag() {
      self.cpu.bus.get_mut_cartridge().clear_irq_flag();
      self.cpu.irq();
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

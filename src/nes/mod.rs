use std::{fs, thread};
use std::borrow::Borrow;
use std::cell::{RefCell, RefMut};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::time::{Duration, Instant};

use glutin::event::{KeyboardInput, WindowEvent};
use glutin::event::{ElementState::{Pressed, Released}, VirtualKeyCode::{A, Down, Escape, Left, R, Right, S, Space, Up, X, Z}};
use glutin::event_loop::ControlFlow;
use glutin::platform::run_return::EventLoopExtRunReturn;
use image::{ImageBuffer, Rgb};
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::texture::{GenMipmaps, Sampler};

use crate::apu::Apu;
use crate::apu::audio_stream::AudioStream;
use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::gfx::WindowContext;
use crate::nes::constants::{KeyboardCommand, REFRESH_RATE, SCREEN_RES_X, SCREEN_RES_Y};
use crate::nes::controller::Controller;
use crate::nes::debug_view::DebugView;
use crate::ppu::{Ppu, PpuState, registers::Registers};

pub mod controller;
pub mod constants;
mod debug_view;

pub type OffScreenBuffer = [[u8; 3]; (SCREEN_RES_X * SCREEN_RES_Y) as usize];

const FRAME_DURATION: Duration = Duration::from_millis((REFRESH_RATE * 1000.0) as u64);

pub struct Nes {
  audio_stream: AudioStream,
  apu: Rc<RefCell<Apu>>,
  cpu: Cpu,
  ppu: Ppu,
  system_cycles: u32,
  window_context: WindowContext,
  controller: Rc<RefCell<Controller>>,
  image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
  off_screen_pixels: Rc<RefCell<OffScreenBuffer>>,
  memory_hash: u64,
  dbg_view: Option<DebugView>,
  is_dbg: bool,
  is_paused: bool,
}

impl Nes {
  pub fn new(rom_file: &str, is_dbg: bool) -> Self {
    let rom_bytes = fs::read(rom_file).expect("Rom file read error");

    let cartridge = Cartridge::new(rom_bytes);
    let cart = Rc::new(RefCell::new(cartridge));

    let window_context = WindowContext::new();

    let registers = Rc::new(RefCell::new(Registers::new(cart.clone())));

    let controller = Rc::new(RefCell::new(Controller::new()));

    let audio_stream = AudioStream::new();

    let apu = Rc::new(RefCell::new(Apu::new()));

    let bus = Bus::new(cart, registers.clone(), controller.clone(), apu.clone());

    let cpu = Cpu::new(bus);

    let off_screen: OffScreenBuffer = [[0u8; 3]; (SCREEN_RES_X * SCREEN_RES_Y) as usize];
    let off_screen_pixels = Rc::new(RefCell::new(off_screen));

    let ppu = Ppu::new(registers, off_screen_pixels.clone());
    let system_cycles = 0;
    let image_buffer = ImageBuffer::new(SCREEN_RES_X, SCREEN_RES_Y);

    let is_paused = false;
    let memory_hash = 0;
    let dbg_view = if is_dbg { Some(DebugView::new(64, 16)) } else { None };

    Nes {
      audio_stream,
      apu,
      cpu,
      ppu,
      system_cycles,
      window_context,
      controller,
      image_buffer,
      off_screen_pixels,
      memory_hash,
      dbg_view,
      is_dbg,
      is_paused,
    }
  }

  #[inline]
  fn get_apu(&mut self) -> RefMut<Apu> {
    self.apu.borrow_mut()
  }

  #[inline]
  fn get_off_screen_pixels(&mut self) -> RefMut<OffScreenBuffer> {
    self.off_screen_pixels.borrow_mut()
  }

  pub fn render_loop(&mut self) {
    let mut last_time = Instant::now();

    let mut keyboard_state = None;
    // 0x80 | 0x40 | 0x20 | 0x10 | 0x08 | 0x04 | 0x02 | 0x01 == 0xFF
    let mut key_map: [bool; 8] = [false, false, false, false, false, false, false, false];

    let mut poll_input = false;

    #[inline]
    fn update_key_map(key_map: &mut [bool; 8], idx: usize, state: bool) {
      if let Some(val) = key_map.get_mut(idx) {
        *val = state
      }
    }

    'app: loop {
      if poll_input {
        poll_input = false;
        let is_paused = self.is_paused;
        self.window_context.event_loop.run_return(|event, _, control_flow| {

          *control_flow = ControlFlow::Wait;

          if let glutin::event::Event::MainEventsCleared = &event {
            *control_flow = ControlFlow::Exit;
          }

          if let glutin::event::Event::WindowEvent { event, .. } = event {
            match event {
              WindowEvent::CloseRequested | WindowEvent::Destroyed => { keyboard_state = Some(KeyboardCommand::Exit) }
              WindowEvent::KeyboardInput { input, .. } => {
                match input {
                  KeyboardInput { state: Released, virtual_keycode: Some(Escape), .. } => {
                    keyboard_state = Some(KeyboardCommand::Exit);
                  }
                  KeyboardInput { state: Released, virtual_keycode: Some(Space), .. } => {
                    if is_paused {
                      keyboard_state = Some(KeyboardCommand::Continue);
                    } else {
                      keyboard_state = Some(KeyboardCommand::Pause);
                    }
                  }
                  KeyboardInput { state, virtual_keycode: Some(X), .. } => update_key_map(&mut key_map, 0, state == Pressed),
                  KeyboardInput { state, virtual_keycode: Some(Z), .. } => update_key_map(&mut key_map, 1, state == Pressed),
                  KeyboardInput { state, virtual_keycode: Some(A), .. } => update_key_map(&mut key_map, 2, state == Pressed),
                  KeyboardInput { state, virtual_keycode: Some(S), .. } => update_key_map(&mut key_map, 3, state == Pressed),
                  KeyboardInput { state, virtual_keycode: Some(Up), .. } => update_key_map(&mut key_map, 4, state == Pressed),
                  KeyboardInput { state, virtual_keycode: Some(Down), .. } => update_key_map(&mut key_map, 5, state == Pressed),
                  KeyboardInput { state, virtual_keycode: Some(Left), .. } => update_key_map(&mut key_map, 6, state == Pressed),
                  KeyboardInput { state, virtual_keycode: Some(Right), .. } => update_key_map(&mut key_map, 7, state == Pressed),
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
          Some(KeyboardCommand::Pause) => self.is_paused = true,
          Some(KeyboardCommand::Continue) => self.is_paused = false,
          Some(KeyboardCommand::Exit) => break 'app,
          Some(KeyboardCommand::Reset) => {
            self.cpu.reset();
            self.ppu.reset();
            self.get_apu().reset();
          }
          Some(KeyboardCommand::Resize) => self.window_context.resize = true,
          _ => {}
        }
        self.controller.borrow_mut().update_buttons(key_map);
      }

      if !self.is_paused {
        self.clock();
      }

      if self.ppu.is_frame_ready || self.is_paused  {
        if keyboard_state == Some(KeyboardCommand::Resize) {
          self.window_context.resize = true;
        }
        self.render_screen();
        self.ppu.is_frame_ready = false;

        if let Some(delay) = FRAME_DURATION.checked_sub(last_time.elapsed()) {
          thread::sleep(delay);
        }
        poll_input = true;
        last_time = Instant::now();
      }
    } // app loop
  }

  fn draw_ram(
    &mut self,
    addr: usize) {
    let mut hasher = DefaultHasher::new();

    let memory = self.cpu.bus_mut_read_dbg_u8(addr, 0x400);
    memory.hash(&mut hasher);
    if self.memory_hash != hasher.finish() {
      if let Some(dbg) = self.dbg_view.as_mut() {
        dbg.send_memory_slice(memory.to_vec());
      }

      hasher = DefaultHasher::new();
      memory.hash(&mut hasher);
      self.memory_hash = hasher.finish();
    }
  }

  fn clock(&mut self) {
    let curr_system_cycles = self.system_cycles;

    let state = self.ppu.clock();

    if state == PpuState::Render {
      self.update_image_buffer();
    }

    if (curr_system_cycles % 3) == 0 {
      if !self.cpu.bus.dma_transfer {
        self.get_apu().step(curr_system_cycles);
        self.cpu.clock(curr_system_cycles);
        if self.is_dbg {
          self.draw_ram(0x0000);
        }
      } else if self.cpu.bus.dma_transfer {
        self.flush_audio_samples();
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

  fn flush_audio_samples(&mut self) {
    let b = self.get_apu().buf.to_vec();
    self.audio_stream.send_audio_buffer(b);
    self.get_apu().buf.clear();
  }


  fn update_image_buffer(&mut self) {
    let pixel_buffer = *self.get_off_screen_pixels();
    for (x, y, pixel) in self.image_buffer.enumerate_pixels_mut() {
      let p = pixel_buffer[y as usize * 256 + x as usize];
      *pixel = Rgb(p);
    }

    self.window_context.texture
      .upload_raw(GenMipmaps::No, &self.image_buffer)
      .expect("Texture update error");
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

    let mut builder = self.window_context.surface.new_pipeline_gate();
    let texture = &mut self.window_context.texture;
    let program = &mut self.window_context.program;
    let copy_program = &mut self.window_context.copy_program;
    let texture_vertices = &self.window_context.texture_vertices;
    let background = &self.window_context.background;

    let mut render = builder.pipeline(
      &self.window_context.front_buffer,
      &PipelineState::default(),
      |_, mut shd_gate| {
        shd_gate.shade(program, |_, _, mut rdr_gate| {
          rdr_gate.render(&RenderState::default(), |mut tess_gate| {
            tess_gate.render(texture_vertices)
          })
        })
      },
    ).assume();

    if render.is_err() {
      panic!("render error");
    }

    render = builder.pipeline(
      &self.window_context.back_buffer,
      &PipelineState::default(),
      |pipeline, mut shd_gate| {
        let bound_texture = pipeline.bind_texture(texture)?;

        shd_gate.shade(copy_program, |mut iface, uni, mut rdr_gate| {
          iface.set(&uni.texture, bound_texture.binding());
          rdr_gate.render(&RenderState::default(), |mut tess_gate| {
            tess_gate.render(background)
          })
        })
      },
    ).assume();

    if render.is_ok() {
      self.window_context.surface.swap_buffers();
    } else {
      panic!("swap buffers error");
    }
  }

  pub fn reset(&mut self) {
    self.cpu.reset();
    self.ppu.reset();
    self.off_screen_pixels.replace([[0u8; 3]; 256 * 240]);
    self.system_cycles = 0;
  }
}

use std::{fs};
use std::borrow::Borrow;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

use std::rc::Rc;

use std::time;
use std::time::{Duration, Instant};

use glutin::event::{KeyboardInput, WindowEvent};
use glutin::event::{ElementState::{Pressed, Released}, VirtualKeyCode::{A, Down, Escape, Left, R, Right, S, Up, X, Z}};
use glutin::event_loop::ControlFlow;
use image::{ImageBuffer, Rgb};
use luminance::context::{GraphicsContext};
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::texture::{GenMipmaps, Sampler};



// use crate::apu::audio_stream::AudioStream;
use crate::apu::Apu;
use crate::apu::audio_stream::AudioStream;
use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::gfx::WindowContext;
use crate::nes::constants::{KeyboardCommand, KeyCode, SCREEN_RES_X, SCREEN_RES_Y};
use crate::ppu::{Ppu, PpuState, registers::Registers};
use glutin::platform::run_return::EventLoopExtRunReturn;

pub mod constants;

pub type OffScreenBuffer = [[u8; 3]; (SCREEN_RES_X * SCREEN_RES_Y) as usize];

pub struct Nes {
  start_time: Instant,
  audio_stream: AudioStream,
  apu: Rc<RefCell<Apu>>,
  cpu: Cpu,
  ppu: Ppu,
  system_cycles: u32,
  window_context: WindowContext,
  controller: Rc<RefCell<[u8; 2]>>,
  image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
  // pub texture: Texture<GL33, Dim2, NormRGB8UI>,
  off_screen_pixels: Rc<RefCell<OffScreenBuffer>>,
}

impl Nes {
  pub fn new(rom_file: &str) -> Self {
    let rom_bytes = fs::read(rom_file).expect("Rom file read error");

    let cartridge = Cartridge::new(rom_bytes);
    let cart = Rc::new(RefCell::new(cartridge));
    let c = [0u8; 2];

    let window_context = WindowContext::new();

    let registers = Rc::new(RefCell::new(Registers::new(cart.clone())));

    let controller = Rc::new(RefCell::new(c));

    let audio_stream = AudioStream::new();

    let start_time = Instant::now();

    let apu = Rc::new(RefCell::new(Apu::new()));

    let bus = Bus::new(cart, registers.clone(), controller.clone(), apu.clone());

    let cpu = Cpu::new(bus);

    let off_screen: OffScreenBuffer = [[0u8; 3]; (SCREEN_RES_X * SCREEN_RES_Y) as usize];
    let off_screen_pixels = Rc::new(RefCell::new(off_screen));

    let ppu = Ppu::new(registers, off_screen_pixels.clone());
    let system_cycles = 0;
    let image_buffer = ImageBuffer::new(SCREEN_RES_X, SCREEN_RES_Y);

    Nes {
      audio_stream,
      start_time,
      apu,
      cpu,
      ppu,
      system_cycles,
      window_context,
      controller,
      image_buffer,
      off_screen_pixels,
    }
  }

  pub fn init(&mut self) {
    self.reset();
  }

  #[inline]
  fn get_controller(&mut self) -> RefMut<[u8; 2]> {
    self.controller.borrow_mut()
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
          self.ppu.reset();
          self.get_apu().reset();
        }
        self.clock();
      } else {
        std::thread::sleep(Duration::from_micros(100));
      }
    } // app loop
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

      let since = self.start_time.elapsed();
      println!("{} - {}", self.get_apu().buf.len(), since.as_millis());
      self.start_time = Instant::now();
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
          // iface.texture.update(&bound_texture);
          rdr_gate.render(&RenderState::default(), |mut tess_gate| {
            tess_gate.render(background)
          })
        })
      },
    ).assume();

    if render.is_ok() {
      self.window_context.surface.swap_buffers();
    } else {
      panic!("boom");
    }
  }

  fn reset(&mut self) {
    self.cpu.reset();
    self.ppu.reset();
    self.off_screen_pixels.replace([[0u8; 3]; 256 * 240]);
    self.system_cycles = 0;
  }
}

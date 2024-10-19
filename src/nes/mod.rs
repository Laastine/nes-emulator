use std::{fs, process, thread};
use std::cell::{RefCell, RefMut};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::time::{Duration, Instant};

use gilrs::{Event, EventType, Gilrs, GilrsBuilder};
use gilrs::Button::{DPadDown, DPadLeft, DPadRight, DPadUp, East, Select, South, Start};
use gilrs::ev::filter::{Filter, Repeat};
use glium::uniform;
use image::{ImageBuffer, Rgb};
use winit::event::WindowEvent;
// use luminance::context::GraphicsContext;
// use luminance::framebuffer::Framebuffer;
// use luminance::pipeline::PipelineState;
// use luminance::render_state::RenderState;
// use luminance::texture::{Sampler, TexelUpload};
use glium::Surface;
use crate::apu::Apu;
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

fn init_controller() -> Gilrs {
  match GilrsBuilder::new().set_update_state(false).build() {
    Ok(g) => g,
    Err(gilrs::Error::NotImplemented(g)) => {
      eprintln!("Current platform is not supported");

      g
    }
    Err(e) => {
      eprintln!("Failed to create gilrs context: {}", e);
      process::exit(-1);
    }
  }
}

pub struct Nes {
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
  gilrs: Gilrs,
  input_filter: Repeat,
}

impl Nes {
  pub fn new(rom_file: &str, is_dbg: bool) -> Self {
    let rom_bytes = fs::read(rom_file).expect("Rom file read error");

    let cartridge = Cartridge::new(rom_bytes);
    let cart = Rc::new(RefCell::new(cartridge));

    let window_context = WindowContext::new();

    let controller = Rc::new(RefCell::new(Controller::new()));

    let apu = Rc::new(RefCell::new(Apu::new()));

    let registers = Rc::new(RefCell::new(Registers::new(cart.clone())));
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

    let gilrs = init_controller();

    let input_filter = Repeat::new();

    Nes {
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
      gilrs,
      input_filter,
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

        while let Some(ev) = self.gilrs.next_event().filter_ev(&self.input_filter, &mut self.gilrs) {
          self.gilrs.update(&ev);
          match ev {
            Event { event: EventType::ButtonChanged(East, val, _), .. } => update_key_map(&mut key_map, 0, val > 0.0),
            Event { event: EventType::ButtonChanged(South, val, _), .. } => update_key_map(&mut key_map, 1, val > 0.0),
            Event { event: EventType::ButtonChanged(Select, val, _), .. } => update_key_map(&mut key_map, 2, val > 0.0),
            Event { event: EventType::ButtonChanged(Start, val, _), .. } => update_key_map(&mut key_map, 3, val > 0.0),
            Event { event: EventType::ButtonChanged(DPadUp, val, _), .. } => update_key_map(&mut key_map, 4, val > 0.0),
            Event { event: EventType::ButtonChanged(DPadDown, val, _), .. } => update_key_map(&mut key_map, 5, val > 0.0),
            Event { event: EventType::ButtonChanged(DPadLeft, val, _), .. } => update_key_map(&mut key_map, 6, val > 0.0),
            Event { event: EventType::ButtonChanged(DPadRight, val, _), .. } => update_key_map(&mut key_map, 7, val > 0.0),
            _ => {}
          }
        }


        match keyboard_state {
          Some(KeyboardCommand::Pause) => self.is_paused = true,
          Some(KeyboardCommand::Continue) => self.is_paused = false,
          Some(KeyboardCommand::Exit) => break 'app,
          Some(KeyboardCommand::Reset) => {
            self.cpu.reset();
            self.ppu.reset();
            self.get_apu().reset();
          }
          // Some(KeyboardCommand::Resize) => self.window_context.resize = true,
          _ => {}
        }
        self.controller.borrow_mut().update_buttons(key_map);
      }

      if !self.is_paused {
        self.clock();
      }
      if self.ppu.is_frame_ready || self.is_paused {
        if keyboard_state == Some(KeyboardCommand::Resize) {
          // self.window_context.resize = true;
        }
        self.render_screen();
        self.ppu.is_frame_ready = false;

        if let Some(delay) = FRAME_DURATION.checked_sub(last_time.elapsed()) {
          thread::sleep(delay);
        }
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
        self.get_apu().flush_samples();
        self.system_cycles = self.system_cycles.wrapping_add(self.cpu.bus.oam_dma_access(self.system_cycles));
      }
    }

    if self.ppu.nmi {
      self.ppu.nmi = false;
      self.cpu.nmi();
    }

    if self.cpu.bus.get_cartridge().irq_flag() {
      self.cpu.bus.get_mut_cartridge().clear_irq_flag();
      self.cpu.irq();
    }

    self.system_cycles = self.system_cycles.wrapping_add(1);
  }

  fn update_image_buffer(&mut self) {
    let pixel_buffer = *self.get_off_screen_pixels();
    for (x, y, pixel) in self.image_buffer.enumerate_pixels_mut() {
      let p = pixel_buffer[y as usize * 256 + x as usize];
      *pixel = Rgb(p);
    }
    self.window_context.update_image_buffer(&self.image_buffer)
  }

  fn render_screen(&mut self) {
    let mut target = self.window_context.display.draw();
    target.clear_color(0.0, 0.0, 0.0, 1.0);

    let uniforms = uniform! {
                        matrix: [
                            [1.0, 0.0, 0.0, 0.0],
                            [0.0, 1.0, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [0.0, 0.0, 0.0, 1.0f32],
                        ],
                        tex: &self.window_context.texture,
                    };

    target.draw(&self.window_context.vertex_buffer, &self.window_context.indices, &&self.window_context.program, &uniforms,
                &Default::default()).unwrap();
    target.finish().unwrap();
  }

  pub fn reset(&mut self) {
    self.cpu.reset();
    self.ppu.reset();
    self.off_screen_pixels.replace([[0u8; 3]; 256 * 240]);
    self.system_cycles = 0;
  }
}

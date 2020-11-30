use glutin::dpi::PhysicalSize;
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::pixel::{NormUnsigned, RGBA32F};
use luminance::shader::{BuiltProgram, Program, Uniform};
use luminance::tess::{Mode, Tess, TessBuilder};
use luminance::texture::{Dim2, Sampler};
use luminance_derive::UniformInterface;
use luminance_gl::GL33;
use luminance_glutin::GlutinSurface;

use crate::gfx::gxf_util::{Semantics, VertexColor, VertexData, VertexPosition};
use crate::nes::constants::{SCREEN_HEIGHT, SCREEN_RES_X, SCREEN_RES_Y, SCREEN_WIDTH};
use luminance::pipeline::TextureBinding;
use glutin::event_loop::EventLoop;

mod gxf_util;

const SHADER_VERT: &str = include_str!("emulator.v.glsl");
const SHADER_FRAG: &str = include_str!("emulator.f.glsl");
const COPY_VS: &str = include_str!("copy.v.glsl");
const COPY_FS: &str = include_str!("copy.f.glsl");

const VERTICES: [VertexData; 4] = [
  VertexData {
    pos: VertexPosition::new([-0.5, -0.5]),
    uv: VertexColor::new([1.0, 0.0, 0.0]),
  },
  VertexData {
    pos: VertexPosition::new([0.5, -0.5]),
    uv: VertexColor::new([0.0, 1.0, 0.0]),
  },
  VertexData {
    pos: VertexPosition::new([0.5, 0.5]),
    uv: VertexColor::new([0.0, 0.0, 1.0]),
  },
  VertexData {
    pos: VertexPosition::new([-0.5, 0.5]),
    uv: VertexColor::new([1.0, 1.0, 0.0]),
  },
];

const INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];

#[derive(UniformInterface)]
pub struct ShaderInterface {
  #[uniform(unbound, name = "source_texture")]
  pub(crate) texture: Uniform<TextureBinding<Dim2, NormUnsigned>>,
}

pub struct WindowContext {
  pub copy_program: Program<GL33, (), (), ShaderInterface>,
  pub program: Program<GL33, Semantics, (), ()>,
  pub back_buffer: Framebuffer<GL33, Dim2, (), ()>,
  pub front_buffer: Framebuffer<GL33, Dim2, RGBA32F, ()>,
  pub surface: GlutinSurface,
  pub resize: bool,
  pub background: Tess<GL33, ()>,
  pub texture_vertices: Tess<GL33, VertexData, u32>,
  pub event_loop: EventLoop<()>
}

impl WindowContext {
  pub fn new() -> WindowContext {
    let (mut surface, event_loop) = GlutinSurface::new_gl33_from_builders(
      |_, window_builder| {
        window_builder
          .with_title("NES-emulator")
          .with_inner_size(PhysicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT))
      },
      |_, context_builder| context_builder.with_double_buffer(Some(true)))
      .expect("Glutin surface create");

    let program = surface
      .new_shader_program::<Semantics, (), ()>()
      .from_strings(SHADER_VERT, None, None, SHADER_FRAG)
      .expect("Program create error")
      .ignore_warnings();

    let BuiltProgram {
      program: copy_program,
      warnings,
    } = surface
      .new_shader_program::<(), (), ShaderInterface>()
      .from_strings(COPY_VS, None, None, COPY_FS)
      .expect("copy program creation");

    for warning in &warnings {
      eprintln!("copy shader warning: {:?}", warning);
    }

    let texture_vertices = surface
      .new_tess()
      .set_vertices(&VERTICES[..])
      .set_indices(INDICES)
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    let background = TessBuilder::new(&mut surface)
      .set_vertex_nb(4)
      .set_mode(Mode::TriangleFan)
      .build()
      .unwrap();

    let back_buffer = surface.back_buffer().unwrap();
    let front_buffer = surface.new_framebuffer::<Dim2, RGBA32F, ()>([SCREEN_RES_X as u32, SCREEN_RES_Y as u32], 0, Sampler::default())
      .expect("Frame buffer create error");

    let resize = false;

    WindowContext {
      copy_program,
      event_loop,
      program,
      surface,
      back_buffer,
      front_buffer,
      resize,
      background,
      texture_vertices,
    }
  }
}

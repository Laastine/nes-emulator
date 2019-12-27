use luminance::framebuffer::Framebuffer;
use luminance::pipeline::BoundTexture;
use luminance::pixel::{NormUnsigned, RGBA32F};
use luminance::shader::program::{BuiltProgram, Program, Uniform};
use luminance::tess::{Mode, Tess, TessBuilder};
use luminance::texture::{Dim2, Flat};
use luminance_derive::UniformInterface;
use luminance_glutin::{GlutinSurface, Surface, WindowDim, WindowOpt};

use crate::gfx::gxf_util::{Semantics, VertexColor, VertexData, VertexPosition};
use crate::nes::constants::{SCREEN_HEIGHT, SCREEN_RES_X, SCREEN_RES_Y, SCREEN_WIDTH};

mod gxf_util;
pub mod texture;

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
  pub(crate) texture: Uniform<&'static BoundTexture<'static, Flat, Dim2, NormUnsigned>>,
}

pub struct WindowContext {
  pub copy_program: Program<(), (), ShaderInterface>,
  pub program: Program<Semantics, (), ()>,
  pub back_buffer: Framebuffer<Flat, Dim2, (), ()>,
  pub front_buffer: Framebuffer<Flat, Dim2, RGBA32F, ()>,
  pub surface: GlutinSurface,
  pub resize: bool,
  pub background: Tess,
  pub triangle: Tess,
}

impl WindowContext {
  pub fn new() -> WindowContext {
    let mut surface = GlutinSurface::new(
      WindowDim::Windowed(SCREEN_WIDTH, SCREEN_HEIGHT),
      "NES-emulator",
      WindowOpt::default(),
    )
    .expect("Glutin surface create");

    let program = Program::<Semantics, (), ()>::from_strings(None, SHADER_VERT, None, SHADER_FRAG)
      .expect("Program create error")
      .ignore_warnings();

    let BuiltProgram {
      program: copy_program,
      warnings,
    } = Program::<(), (), ShaderInterface>::from_strings(None, COPY_VS, None, COPY_FS)
      .expect("Copy program create");

    for warning in &warnings {
      eprintln!("Copy shader warning: {:?}", warning);
    }

    let texture_vertices = TessBuilder::new(&mut surface)
      .add_vertices(VERTICES)
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
    let front_buffer =
      Framebuffer::<Flat, Dim2, RGBA32F, ()>::new(&mut surface, [SCREEN_RES_X, SCREEN_RES_Y], 0)
        .expect("Framebuffer create error");

    let resize = false;

    WindowContext {
      copy_program,
      program,
      surface,
      back_buffer,
      front_buffer,
      resize,
      background,
      triangle: texture_vertices,
    }
  }
}

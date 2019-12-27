use luminance_derive::{Semantics, Vertex};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
  #[sem(name = "co", repr = "[f32; 2]", wrapper = "VertexPosition")]
  Position,
  #[sem(name = "color", repr = "[f32; 3]", wrapper = "VertexColor")]
  Color,
  #[sem(
    name = "position",
    repr = "[f32; 2]",
    wrapper = "VertexInstancePosition"
  )]
  InstancePosition,
  #[sem(name = "weight", repr = "f32", wrapper = "VertexWeight")]
  Weight,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
pub struct VertexData {
  pub pos: VertexPosition,
  pub uv: VertexColor,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics", instanced = "true")]
pub struct Instance {
  pub pos: VertexInstancePosition,
  pub w: VertexWeight,
}

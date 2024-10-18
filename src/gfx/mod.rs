use glium::{implement_vertex, Display, Texture2d, Vertex, VertexBuffer};
use glium::texture::Texture2dArray;
use glium::vertex::VertexBufferAny;
use glutin::surface::WindowSurface;
use image::ImageBuffer;
use crate::nes::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};

const vertex_shader_src: &str = r#"
        #version 140

        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;

        uniform mat4 matrix;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;
const fragment_shader_src: &str = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;

        uniform sampler2D tex;

        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;

pub struct WindowContext {
    pub texture: Texture2d,
    pub vertex_buffer: VertexBufferAny,
    pub program: glium::Program,
    window: winit::window::Window,
    pub event_loop: winit::event_loop::EventLoop<()>,
    pub indices: glium::index::NoIndices,
    pub display: Display<WindowSurface>,
}

impl WindowContext {
    pub fn new() -> Self {

        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }
        implement_vertex!(Vertex, position, tex_coords);
        // We've changed our shape to a rectangle so the image isn't distorted.
        let shape = vec![
            Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 1.0] },

            Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5,  0.5], tex_coords: [0.0, 1.0] },
            Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0] },
        ];


        let event_loop = winit::event_loop::EventLoopBuilder::new().build().unwrap();
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title("NES-emulator")
            .with_inner_size(SCREEN_WIDTH, SCREEN_HEIGHT)
            .build(&event_loop);

        let texture = glium::Texture2d::empty(&display, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();

        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let vertex_buffer = glium::VertexBuffer::dynamic(&display, &shape).unwrap();

        let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();
        WindowContext {
            texture,
            vertex_buffer: vertex_buffer.into(),
            program,
            window,
            event_loop,
            indices,
            display,
        }
    }

    pub fn update_image_buffer(&mut self) {

    }
}
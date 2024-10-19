use glium::{implement_vertex, Display, Texture2d};
use glium::texture::RawImage2d;
use glium::vertex::VertexBufferAny;
use glutin::surface::WindowSurface;
use image::{ImageBuffer, Rgb};
use crate::nes::constants::{SCALING_FACTOR, SCREEN_RES_Y, SCREEN_RES_X};

const VERTEX_SHADER_SRC: &str = r#"
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
const FRAGMENT_SHADER_SRC: &str = r#"
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


        let event_loop = winit::event_loop::EventLoopBuilder::new().build();
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title("NES-emulator")
            .with_inner_size(SCREEN_RES_X*SCALING_FACTOR, SCREEN_RES_Y*SCALING_FACTOR)
            .build(&event_loop);

        let texture = glium::Texture2d::empty(&display, SCREEN_RES_X, SCREEN_RES_Y).unwrap();
        // let framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture).unwrap();

        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let vertex_buffer = glium::VertexBuffer::dynamic(&display, &shape).unwrap();

        let program = glium::Program::from_source(&display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();
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

    pub fn update_image_buffer(&mut self, image_buffer: &ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let raw_image = RawImage2d::from_raw_rgb(image_buffer.pixels().flat_map(|Rgb(d)| d.to_vec()).collect(), (SCREEN_RES_X, SCREEN_RES_Y));
        self.texture.write(glium::Rect { left: 0, bottom: 0, width: SCREEN_RES_X, height: SCREEN_RES_Y }, raw_image);
    }
}
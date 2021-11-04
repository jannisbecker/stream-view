#[macro_use]
extern crate glium;

use std::{sync::mpsc, thread};

use glium::{
    glutin::{
        dpi::LogicalSize,
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    index::PrimitiveType,
    texture::{RawImage2d, SrgbTexture2d},
    Display, IndexBuffer, Program, Surface, VertexBuffer,
};
use std::time::{Duration, Instant};

use crate::video::VideoDevice;

mod support;
mod video;
//mod window;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

fn create_vertex_buffer(display: &Display) -> VertexBuffer<Vertex> {
    {
        implement_vertex!(Vertex, position, tex_coords);

        VertexBuffer::new(
            display,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                    tex_coords: [1.0, 0.0],
                },
            ],
        )
        .unwrap()
    }
}
fn create_index_buffer(display: &Display) -> IndexBuffer<u16> {
    IndexBuffer::new(display, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3]).unwrap()
}
fn create_shader_program(display: &Display) -> Program {
    program!(display,
        140 => {
            vertex: "
                #version 140
                uniform mat4 matrix;
                in vec2 position;
                in vec2 tex_coords;
                out vec2 v_tex_coords;
                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",

            fragment: "
                #version 140
                uniform sampler2D tex;
                in vec2 v_tex_coords;
                out vec4 f_color;
                void main() {
                    f_color = texture(tex, v_tex_coords);
                }
            "
        },

        110 => {
            vertex: "
                #version 110
                uniform mat4 matrix;
                attribute vec2 position;
                attribute vec2 tex_coords;
                varying vec2 v_tex_coords;
                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",

            fragment: "
                #version 110
                uniform sampler2D tex;
                varying vec2 v_tex_coords;
                void main() {
                    gl_FragColor = texture2D(tex, v_tex_coords);
                }
            ",
        },

        100 => {
            vertex: "
                #version 100
                uniform lowp mat4 matrix;
                attribute lowp vec2 position;
                attribute lowp vec2 tex_coords;
                varying lowp vec2 v_tex_coords;
                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",

            fragment: "
                #version 100
                uniform lowp sampler2D tex;
                varying lowp vec2 v_tex_coords;
                void main() {
                    gl_FragColor = texture2D(tex, v_tex_coords);
                }
            ",
        },
    )
    .unwrap()
}

fn main() {
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(1);

    thread::spawn(move || {
        let mut video_device = VideoDevice::new();
        video_device.open();

        loop {
            let frame = video_device.grab_frame();
            let _ = tx.send(frame);
        }
    });

    let event_loop = EventLoop::new();
    let display = Display::new(
        WindowBuilder::new().with_inner_size(LogicalSize::new(WIDTH, HEIGHT)),
        ContextBuilder::new().with_vsync(false),
        &event_loop,
    )
    .unwrap();

    let vertex_buffer = create_vertex_buffer(&display);
    let index_buffer = create_index_buffer(&display);
    let program = create_shader_program(&display);

    let wait_time = Duration::from_secs_f32((1 / 165) as f32);

    event_loop.run(move |ev, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + wait_time);

        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            Event::NewEvents(cause) => match cause {
                StartCause::ResumeTimeReached { .. } => (),
                StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let _ = rx.recv().and_then(|frame| {
            let image = RawImage2d::from_raw_rgb_reversed(&frame, (WIDTH, HEIGHT));
            let opengl_texture = SrgbTexture2d::new(&display, image).unwrap();

            // building the uniforms
            let uniforms = uniform! {
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ],
                tex: &opengl_texture
            };

            let mut target = display.draw();
            target
                .draw(
                    &vertex_buffer,
                    &index_buffer,
                    &program,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();

            target.finish().unwrap();

            Ok(())
        });
    });
}

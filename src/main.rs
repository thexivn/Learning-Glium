#[macro_use]
extern crate glium;
extern crate image;

use glium::{
    glutin::{ContextBuilder, Event, EventsLoop, WindowBuilder, WindowEvent},
    index::{NoIndices, PrimitiveType},
    texture::{RawImage2d, Texture2d},
    Display, Program, Surface, VertexBuffer,
};

use std::io::Cursor;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2]
}

fn main() {
    //display
    let mut events_loop = EventsLoop::new();
    let window_builder = WindowBuilder::new().with_title("Titolino");
    let context_builder = ContextBuilder::new();
    let display = Display::new(window_builder, context_builder, &events_loop).unwrap();
    //image
    let image = image::load(
        Cursor::new(&include_bytes!("../assets/texture/brick_wall.png")[..]),
        image::PNG
    )
    .unwrap()
    .to_rgba();

    let dimensions = image.dimensions();
    //rgb case errors
    let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dimensions);
    let texture = Texture2d::new(&display, image).unwrap();
    println!("QUi");

    implement_vertex!(Vertex, position, tex_coords);

    let vertex1 = Vertex {
        position: [-0.5, -0.5],
        tex_coords: [0.0, 0.0]
    };
    let vertex2 = Vertex {
        position: [0.0, 0.5],
        tex_coords: [0.0, 1.0]
    };
    let vertex3 = Vertex {
        position: [0.5, -0.25],
        tex_coords: [1.0, 0.0]
    };
    let shape = vec![vertex1, vertex2, vertex3];

    let vertex_buffer = VertexBuffer::new(&display, &shape).unwrap();
    let indices = NoIndices(PrimitiveType::TrianglesList);

    //Note that it is important to write matrix * vertex and not vertex * matrix.
    //Matrix operations produce different results depending on the order.
    let vertex_shader_src = r#"
        #version 140

        in vec2 tex_coords;
        in vec2 position;
        out vec2 v_text_coords;

        uniform mat4 matrix;

        void main() {
            v_text_coords = tex_coords;
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec2 v_text_coords;
        out vec4 color;

        uniform sampler2D tex;

        void main() {
            
            color = texture(tex, v_text_coords);
        }
    "#;

    let program =
        Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let mut transform: f32 = -0.5;
    let mut closed = false;
    while !closed {
        transform += 0.002;
        if transform > 0.5 {
            transform = -0.5;
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        //move left to right
        let uniform = uniform! {
            matrix : [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [transform, 0.0, 0.0, 1.0],
            ],
            tex: &texture,
        };
        //let uniform = uniform! {
        //    matrix: [
        //        [ transform.cos(), transform.sin(), 0.0, 0.0],
        //        [-transform.sin(), transform.cos(), 0.0, 0.0],
        //        [0.0, 0.0, 1.0, 0.0],
        //        [0.0, 0.0, 0.0, 1.0],
        //    ]
        //};

        target
            .draw(
                &vertex_buffer,
                &indices,
                &program,
                &uniform,
                &Default::default(),
            )
            .unwrap();

        target.finish().unwrap();

        events_loop.poll_events(|ev| match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => closed = true,
                _ => (),
            },
            _ => (),
        });
    }
}

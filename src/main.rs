#[macro_use]
extern crate glium;

use glium::{
    glutin::{ContextBuilder, Event, EventsLoop, WindowBuilder, WindowEvent},
    index::{NoIndices, PrimitiveType},
    Display, Program, Surface, VertexBuffer,
};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

fn main() {
    let mut events_loop = EventsLoop::new();
    let window_builder = WindowBuilder::new().with_title("Titolino");
    let context_builder = ContextBuilder::new();
    let display = Display::new(window_builder, context_builder, &events_loop).unwrap();

    implement_vertex!(Vertex, position);

    let vertex1 = Vertex {
        position: [-0.5, -0.5],
    };
    let vertex2 = Vertex {
        position: [0.0, 0.5],
    };
    let vertex3 = Vertex {
        position: [0.5, -0.5],
    };
    let shape = vec![vertex1, vertex2, vertex3];

    let vertex_buffer = VertexBuffer::new(&display, &shape).unwrap();
    let indices = NoIndices(PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        uniform float t;

        void main() {
            vec2 pos = position;
            pos.x += t;
            gl_Position = vec4(pos, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        out vec4 color;

        void main() {
            color = vec4(1.0, 0.0, 0.0, 1.0);
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

        target
            .draw(
                &vertex_buffer,
                &indices,
                &program,
                &uniform! {t: transform},
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

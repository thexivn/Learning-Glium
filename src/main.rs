#[macro_use]
extern crate glium;

use glium::{
    glutin::{ContextBuilder, Event, EventsLoop, WindowBuilder, WindowEvent},
    index::PrimitiveType,
    texture::{RawImage2d, Texture2d},
    Display, IndexBuffer, Program, Surface, VertexBuffer,
};

use ply_rs::{
    parser::Parser,
    ply::{Property, PropertyAccess},
};

use std::io::Cursor;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: (f32, f32, f32),
    tex_coords: [f32; 2],
}

#[derive(Copy, Clone, Debug)]
struct Normal {
    normal: (f32, f32, f32),
}

#[derive(Copy, Clone, Debug)]
struct Shape {
    vertex : Vertex,
    normali : Normal
}

#[derive(Debug)]
struct Face {
    vertex_index: Vec<u32>,
}

impl Vertex {
    fn new() -> Self {
        Vertex {
            position: (0.0, 0.0, 0.0),
            tex_coords: [0.0, 0.0],
        }
    }
}

impl Normal {
    fn new() -> Self {
        Normal {
            normal: (0.0, 0.0, 0.0),
        }
    }
}

impl PropertyAccess for Shape {
    fn new() -> Self {
        Shape {
            vertex: Vertex::new(),
            normali: Normal::new()
        }
    }

    fn set_property(&mut self, key: String, property: Property) {
        match (key.as_ref(), property) {
            ("x", Property::Float(v)) => self.vertex.position.0 = v,
            ("y", Property::Float(v)) => self.vertex.position.1 = v,
            ("z", Property::Float(v)) => self.vertex.position.2 = v,
            ("nx", Property::Float(v)) => self.normali.normal.0 = v,
            ("ny", Property::Float(v)) => self.normali.normal.1 = v,
            ("nz", Property::Float(v)) => self.normali.normal.2 = v,
            (_, _) => (),
        }
    }
}

impl PropertyAccess for Face {
    fn new() -> Self {
        Face {
            vertex_index: Vec::new(),
        }
    }
    fn set_property(&mut self, key: String, property: Property) {
        match (key.as_ref(), property) {
            ("vertex_indices", Property::ListUInt(vec)) => self.vertex_index = vec,
            (k, _) => println!("Face: Unexpected key/value combination: key:{}", k),
        }
    }
}

fn main() {
    //display
    let mut events_loop = EventsLoop::new();
    let window_builder = WindowBuilder::new().with_title("Titolino");
    let context_builder = ContextBuilder::new();
    let display = Display::new(window_builder, context_builder, &events_loop).unwrap();

    let file = std::fs::File::open("assets/monkey.ply").unwrap();
    let mut file = std::io::BufReader::new(file);
    println!("Monkey OK");

    let shape_parser = Parser::<Shape>::new();
    let face_parser = Parser::<Face>::new();
    let header = shape_parser.read_header(&mut file).unwrap();
    println!("header OK");

    let mut shape_list = Vec::new();
    let mut face_list = [0u32; 968*3];
    for (_ignore_key, element) in &header.elements {
        match element.name.as_ref() {
            "vertex" => {
                shape_list = shape_parser
                    .read_payload_for_element(&mut file, &element, &header)
                    .unwrap();
                println!("shape OK");
            }
            "face" => {
                let tmp = face_parser
                    .read_payload_for_element(&mut file, &element, &header)
                    .unwrap();
                println!("Faces OK");
                let mut i = 0;
                for face in tmp.into_iter() {
                    
                    for index in face.vertex_index.into_iter() {
                        face_list[i] = index;
                        i = i + 1;
                    }
                }
            }
            _ => panic!("Enexpeced element!"),
        }
    }

    implement_vertex!(Vertex, position, tex_coords);
    implement_vertex!(Normal, normal);

    let vertex_list: Vec<Vertex> = shape_list.clone().into_iter().map(|shape| shape.vertex).collect();
    let normal_list: Vec<Normal> = shape_list.into_iter().map(|shape| shape.normali).collect();

    let position = VertexBuffer::new(&display, &vertex_list ).unwrap();
    let normals = VertexBuffer::new(&display, &normal_list).unwrap();
    let indices = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &face_list).unwrap();

    //Note that it is important to write matrix * vertex and not vertex * matrix.
    //Matrix operations produce different results depending on the order.
    let vertex_shader_src = r#"
        #version 150      // updated

        in vec3 position;
        in vec3 normal;
        
        out vec3 v_normal;      // new
        
        uniform mat4 matrix;
        
        void main() {
            v_normal = transpose(inverse(mat3(matrix))) * normal;       // new
            gl_Position = matrix * vec4(position, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec3 v_normal;
        out vec4 color;
        uniform vec3 u_light;
        
        void main() {
            float brightness = dot(normalize(v_normal), normalize(u_light));
            vec3 dark_color = vec3(0.6, 0.0, 0.0);
            vec3 regular_color = vec3(1.0, 0.0, 0.0);
            color = vec4(mix(dark_color, regular_color, brightness), 1.0);
        }
    "#;

    let program =
        Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let mut transform: f32 = -0.5;
    let mut closed = false;
    while !closed {
        transform += 0.02;
        if transform > 100.0 {
            transform = -0.0;
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        //move left to right
        //let uniform = uniform! {
        //    matrix : [
        //        [transform, 0.0, 0.0, 0.0],
        //        [0.0, transform, 0.0, 0.0],
        //        [0.0, 0.0, transform, 0.0],
        //        [0.0, 0.0, 0.0, 1.0f32],
        //    ],
        //    u_light: [-1.0, 0.4, 0.9f32]
        //};
        let uniform = uniform! {
            matrix: [
                [ transform.cos(), transform.sin(), 0.0, 0.0],
                [-transform.sin(), transform.cos(), transform.cos()*0.5, 0.0],
                [transform.sin(), -transform.sin(), -transform.sin(), 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            u_light: [-1.0, 0.4, 0.9f32],
        };

        target
            .draw(
                (&position, &normals),
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

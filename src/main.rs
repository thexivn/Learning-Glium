#[macro_use]
extern crate glium;
mod util;

use glium::{
    draw_parameters::DepthTest,
    glutin::{
        ContextBuilder, Event, EventsLoop, WindowBuilder, WindowEvent, VirtualKeyCode
    },
    index::PrimitiveType,
    Depth, Display, DrawParameters, IndexBuffer, Program, Surface, VertexBuffer,
};

use ply_rs::{
    parser::Parser,
    ply::{Property, PropertyAccess},
};

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
    vertex: Vertex,
    normali: Normal,
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
            normali: Normal::new(),
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
    let display = get_display("Titolino", &events_loop).unwrap();
    let model = load_3d_model("assets/monkey.ply");

    implement_vertex!(Vertex, position, tex_coords);
    implement_vertex!(Normal, normal);

    let vertex_list: Vec<Vertex> = model
        .0
        .clone()
        .into_iter()
        .map(|shape| shape.vertex)
        .collect();
    let normal_list: Vec<Normal> = model.0.into_iter().map(|shape| shape.normali).collect();

    let position = VertexBuffer::new(&display, &vertex_list).unwrap();
    let normals = VertexBuffer::new(&display, &normal_list).unwrap();
    let indices = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &model.1).unwrap();

    //Note that it is important to write matrix * vertex and not vertex * matrix.
    //Matrix operations produce different results depending on the order.
    let vertex_shader_src = r#"
        #version 140

        in vec3 position;
        in vec3 normal;
        
        out vec3 v_normal;
        
        uniform mat4 perspective;
        uniform mat4 view;
        uniform mat4 model;
        
        void main() {
            mat4 modelview = view * model;
            v_normal = transpose(inverse(mat3(modelview))) * normal;
            gl_Position = perspective * modelview * vec4(position, 1.0);
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
    let mut camera_position = [0.0, -3.0, 0.0];
    let mut camera_direction = [0.0, 1.0, 0.0];
    let speed = 0.15;

    while !closed {
        transform += 0.002;
        if transform > 5.0 {
            transform = 1.0;
        }

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        let (w, h) = target.get_dimensions();
        //move left to right
        let uniform = uniform! {
            model : [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 2.0, 0.0, 1.0f32],
            ],
            view: util::view_matrix(&camera_position, &camera_direction, &[0.0, 0.0, 1.0]),
            perspective: util::get_perspective_matrix(w as f32, h as f32),
            u_light: [-1.0, 0.4, 0.9f32]
        };
        //let uniform = uniform! {
        //    matrix: [
        //        [ transform.cos(), transform.sin(), 0.0, 0.0],
        //        [-transform.sin(), transform.cos(), transform.cos()*0.5, 0.0],
        //        [transform.sin(), -transform.sin(), -transform.sin(), 0.0],
        //        [0.0, 0.0, 0.0, 1.0],
        //    ],
        //    u_light: [-1.0, 0.4, 0.9f32],
        //};
        let params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        target
            .draw((&position, &normals), &indices, &program, &uniform, &params)
            .unwrap();

        target.finish().unwrap();

        events_loop.poll_events(|ev| match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => closed = true,
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == glium::glutin::ElementState::Pressed {
                        match input.virtual_keycode {
                            Some(key) => match key {
                                VirtualKeyCode::W => {
                                    camera_position[0] += camera_direction[0] * speed;
                                    camera_position[1] += camera_direction[1] * speed;
                                    camera_position[2] += camera_direction[2] * speed;
                                },
                                VirtualKeyCode::S => {
                                    camera_position[0] -= camera_direction[0] * speed;
                                    camera_position[1] -= camera_direction[1] * speed;
                                    camera_position[2] -= camera_direction[2] * speed;
                                },
                                VirtualKeyCode::Q => camera_position[1] += speed,
                                VirtualKeyCode::E => camera_position[1] -= speed,
                                VirtualKeyCode::D => {
                                    let perpendicular = [-camera_direction[1], camera_direction[0], 0.0];
                                    camera_position[0] += perpendicular[0] * speed;
                                    camera_position[1] += perpendicular[1] * speed;
                                    camera_position[2] += perpendicular[2] * speed;
                                },
                                VirtualKeyCode::A => {
                                    let perpendicular = [camera_direction[1], camera_direction[0], 0.0];
                                    camera_position[0] += perpendicular[0] * speed;
                                    camera_position[1] += perpendicular[1] * speed;
                                    camera_position[2] += perpendicular[2] * speed;
                                },
                                VirtualKeyCode::Up => camera_direction[0] += speed,
                                VirtualKeyCode::Down => camera_direction[0] -= speed,
                                VirtualKeyCode::Left => camera_direction[2] += speed,
                                VirtualKeyCode::Right => camera_direction[2] -= speed,
                                VirtualKeyCode::F1 => println!("Position: {:?}, Direction {:?}", camera_position, camera_direction),
                                _ => (),
                            },
                            None => (),
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        });

    }

    println!("fine");
}

fn get_display(
    titolo: &'static str,
    event_loop: &EventsLoop,
) -> Result<Display, glium::backend::glutin::DisplayCreationError> {
    let window_builder = WindowBuilder::new().with_title(titolo);
    let context_builder = ContextBuilder::new().with_depth_buffer(24);
    let display = Display::new(window_builder, context_builder, event_loop)?;

    Ok(display)
}

fn load_3d_model(file_name: &'static str) -> (Vec<Shape>, [u32; 2904]) {
    let file = std::fs::File::open(file_name).unwrap();
    let mut file = std::io::BufReader::new(file);
    println!("File OK");

    let shape_parser = Parser::<Shape>::new();
    let face_parser = Parser::<Face>::new();
    let header = shape_parser.read_header(&mut file).unwrap();
    println!("header OK");

    let mut shape_list = Vec::new();
    let mut face_list = [0u32; 968 * 3];
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
                    for indx in face.vertex_index.into_iter() {
                        face_list[i] = indx;
                        i += 1;
                    }
                }
            }
            _ => panic!("Enexpeced element!"),
        }
    }

    (shape_list, face_list)
}



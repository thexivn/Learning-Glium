#[macro_use]
extern crate glium;

use glium::{
    draw_parameters::DepthTest,
    glutin::{
        ContextBuilder, Event, EventsLoop, WindowBuilder, WindowEvent,
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

    while !closed {
        transform += 0.002;
        if transform > 5.0 {
            transform = 1.0;
        }

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        
        //move left to right
        let uniform = uniform! {
            model : [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 2.0, 0.0, 1.0f32],
            ],
            view: view_matrix(&camera_position, &camera_direction, &[0.0, 0.0, 1.0]),
            perspective: get_perspective_matrix(&target),
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
                                glium::glutin::VirtualKeyCode::W => {
                                    camera_position[0] += camera_direction[0] * 0.01;
                                    camera_position[1] += camera_direction[1] * 0.01;
                                    camera_position[2] += camera_direction[2] * 0.01;
                                },
                                glium::glutin::VirtualKeyCode::S => {
                                    camera_position[0] -= camera_direction[0] * 0.01;
                                    camera_position[1] -= camera_direction[1] * 0.01;
                                    camera_position[2] -= camera_direction[2] * 0.01;
                                },
                                glium::glutin::VirtualKeyCode::Q => camera_position[1] += 0.01,
                                glium::glutin::VirtualKeyCode::E => camera_position[1] -= 0.01,
                                glium::glutin::VirtualKeyCode::D => camera_position[2] += 0.01,
                                glium::glutin::VirtualKeyCode::A => camera_position[2] -= 0.01,
                                glium::glutin::VirtualKeyCode::Up => camera_direction[0] += 0.01,
                                glium::glutin::VirtualKeyCode::Down => camera_direction[0] -= 0.01,
                                glium::glutin::VirtualKeyCode::Left => camera_direction[2] += 0.01,
                                glium::glutin::VirtualKeyCode::Right => camera_direction[2] -= 0.01,
                                glium::glutin::VirtualKeyCode::F1 => println!("Position: {:?}, Direction {:?}", camera_position, camera_direction),
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

fn get_perspective_matrix(target: &glium::Frame) -> [[f32; 4]; 4] {
    let (width, height) = target.get_dimensions();
    let aspect_ratio = height as f32 / width as f32;

    let fov: f32 = 3.141592 / 3.0;
    let zfar = 1024.0;
    let znear = 0.1;

    let f = 1.0 / (fov / 2.0).tan();

    [
        [f * aspect_ratio, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
        [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0],
    ]
}

//The position of the camera in the scene.
//The direction the camera is facing in scene coordinates.
//The up vector, representing the direction in scene coordinates of the top of the screen.
fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [
        up[1] * f[2] - up[2] * f[1],
        up[2] * f[0] - up[0] * f[2],
        up[0] * f[1] - up[1] * f[0],
    ];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [
        f[1] * s_norm[2] - f[2] * s_norm[1],
        f[2] * s_norm[0] - f[0] * s_norm[2],
        f[0] * s_norm[1] - f[1] * s_norm[0],
    ];

    let p = [
        -position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
        -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
        -position[0] * f[0] - position[1] * f[1] - position[2] * f[2],
    ];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

use cgmath::Vector3;

pub enum Direction{
    X,
    Y,
    Z,
}


pub fn get_perspective_matrix(width: f32,height : f32) -> [[f32; 4]; 4] {
    let aspect_ratio = height / width;

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

///The position of the camera in the scene.
///The direction the camera is facing in scene coordinates.
///The up vector, representing the direction in scene coordinates of the top of the screen.
pub fn view_matrix(position: &Vector3<f32>, direction: &Vector3<f32>, up_dir: Direction) -> [[f32; 4]; 4] {
    let up = get_unit_vector(up_dir);
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
        -position.x * s_norm[0] - position.y * s_norm[1] - position.z * s_norm[2],
        -position.x * u[0] - position.y * u[1] - position.z * u[2],
        -position.x * f[0] - position.y * f[1] - position.z * f[2],
    ];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

fn get_unit_vector(dir : Direction) -> Vector3::<f32> {
    match dir {
        Direction::X => Vector3::new(1.0, 0.0, 0.0),
        Direction::Y => Vector3::new(0.0, 1.0, 0.0),
        Direction::Z => Vector3::new(0.0, 0.0, 1.0),
    }
}
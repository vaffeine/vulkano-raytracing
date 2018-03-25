extern crate cgmath;

use cgmath::InnerSpace;

use input;

use super::std::f32::consts::PI;
use super::std::fmt;

const UP: cgmath::Vector3<f32> = cgmath::Vector3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

#[derive(Debug)]
pub struct Camera {
    position: cgmath::Vector3<f32>,
    view_dir: cgmath::Vector3<f32>,
    fov: [f32; 2],
    yaw: f32,
    pitch: f32,
}

impl Camera {
    pub fn with_position(position: cgmath::Vector3<f32>, fov: [f32; 2]) -> Camera {
        const DEFAULT_YAW: f32 = 0.0;
        const DEFAULT_PITCH: f32 = 0.0;
        Camera {
            position: position,
            view_dir: view_direction(DEFAULT_YAW, DEFAULT_PITCH),
            fov: fov,
            yaw: DEFAULT_YAW,
            pitch: DEFAULT_PITCH,
        }
    }
    pub fn position(&self) -> [f32; 3] {
        self.position.into()
    }
    pub fn view(&self) -> [f32; 3] {
        self.view_dir.into()
    }
    pub fn axises(&self) -> ([f32; 3], [f32; 3]) {
        let horiz_axis = self.view_dir.cross(UP).normalize();
        let vert_axis = horiz_axis.cross(self.view_dir).normalize();
        let right = horiz_axis * scale(self.fov[0]);
        let up = vert_axis * scale(-self.fov[1]);
        (up.into(), right.into())
    }
    pub fn process_keyboard_input(&mut self, keyboard: &input::Keyboard, delta_seconds: f32) {
        const SPEED: f32 = 2.5;
        let relative_speed = SPEED * delta_seconds;
        if keyboard.forward_pressed {
            self.move_forward(relative_speed);
        }
        if keyboard.back_pressed {
            self.move_forward(-relative_speed);
        }
        if keyboard.right_pressed {
            self.strafe(relative_speed);
        }
        if keyboard.left_pressed {
            self.strafe(-relative_speed);
        }
        if keyboard.top_pressed {
            self.move_up(relative_speed);
        }
        if keyboard.bottom_pressed {
            self.move_up(-relative_speed);
        }
    }
    pub fn process_mouse_input(&mut self, delta: (f64, f64)) {
        const SPEED: f32 = 0.005;
        self.add_yaw(-delta.0 as f32 * SPEED);
        self.add_pitch(delta.1 as f32 * SPEED);
    }
    fn move_forward(&mut self, delta: f32) {
        self.position += self.view_dir * delta;
    }
    fn move_up(&mut self, delta: f32) {
        self.position += cgmath::Vector3::new(0.0, delta, 0.0);
    }
    fn strafe(&mut self, delta: f32) {
        self.position += self.view_dir.cross(UP).normalize() * delta;
    }
    fn add_yaw(&mut self, delta: f32) {
        self.yaw = (self.yaw + delta) % (2.0 * PI);
        self.view_dir = view_direction(self.yaw, self.pitch);
    }
    fn add_pitch(&mut self, delta: f32) {
        const PADDING: f32 = PI / 4.0;
        self.pitch = clamp(self.pitch + delta, -PI / 2.0 + PADDING, PI / 2.0 - PADDING);
        self.view_dir = view_direction(self.yaw, self.pitch);
    }
}

impl fmt::Display for Camera {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "pos: [{:.2}, {:.2}, {:.2}], yaw: {:.2}, pitch: {:.2}",
            self.position.x, self.position.y, self.position.z, self.yaw, self.pitch
        )
    }
}

fn view_direction(yaw: f32, pitch: f32) -> cgmath::Vector3<f32> {
    -1.0 * cgmath::Vector3::new(
        yaw.sin() * pitch.cos(),
        pitch.sin(),
        yaw.cos() * pitch.cos(),
    ).normalize()
}

fn scale(fov: f32) -> f32 {
    (0.5 * fov * PI / 180.0).tan()
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.min(max).max(min)
}

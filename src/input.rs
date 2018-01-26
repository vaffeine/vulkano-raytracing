extern crate winit;

use std::mem;

#[derive(Debug)]
pub struct Keyboard {
    pub forward_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub back_pressed: bool,
    pub top_pressed: bool,
    pub bottom_pressed: bool,
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            forward_pressed: false,
            left_pressed: false,
            right_pressed: false,
            back_pressed: false,
            top_pressed: false,
            bottom_pressed: false,
        }
    }

    pub fn handle_keypress(&mut self, event: &winit::KeyboardInput) {
        let pressed = event.state == winit::ElementState::Pressed;
        match event.scancode {
            25 | 111 => self.forward_pressed = pressed,
            38 | 113 => self.left_pressed = pressed,
            40 | 114 => self.right_pressed = pressed,
            39 | 116 => self.back_pressed = pressed,
            50 | 62 => self.top_pressed = pressed,
            37 | 105 => self.bottom_pressed = pressed,
            _ => (),
        };
    }
}

#[derive(Debug)]
pub struct Mouse {
    mouse_delta: (f64, f64),
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse {
            mouse_delta: (0.0, 0.0),
        }
    }
    pub fn handle_mousemove(&mut self, axis: winit::AxisId, value: f64) {
        match axis {
            0 => self.mouse_delta.0 += value,
            1 => self.mouse_delta.1 += value,
            _ => (),
        }
    }
    pub fn fetch_mouse_delta(&mut self) -> (f64, f64) {
        mem::replace(&mut self.mouse_delta, (0.0, 0.0))
    }
}

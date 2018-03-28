extern crate winit;

use std::mem;

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

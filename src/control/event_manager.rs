extern crate winit;

use super::keyboard::Keyboard;
use super::mouse::Mouse;

use std::mem;

pub struct EventManager {
    pub keyboard: Keyboard,
    pub mouse: Mouse,
    done: bool,
    recreate_swapchain: bool,
}

impl EventManager {
    pub fn new() -> EventManager {
        EventManager {
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
            done: false,
            recreate_swapchain: false,
        }
    }

    pub fn process_event(&mut self, ev: winit::Event) {
        match ev {
            winit::Event::WindowEvent {
                event: winit::WindowEvent::Closed,
                ..
            }
            | winit::Event::WindowEvent {
                event:
                    winit::WindowEvent::KeyboardInput {
                        input:
                            winit::KeyboardInput {
                                state: winit::ElementState::Released,
                                scancode: 9,
                                ..
                            },
                        ..
                    },
                ..
            } => self.done = true,
            winit::Event::WindowEvent {
                event: winit::WindowEvent::Resized(_, _),
                ..
            } => self.recreate_swapchain = true,
            winit::Event::WindowEvent {
                event: winit::WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                self.keyboard.handle_keypress(&input);
            }
            winit::Event::DeviceEvent {
                event: winit::DeviceEvent::Motion { axis, value },
                ..
            } => {
                self.mouse.handle_mousemove(axis, value);
            }
            _ => (),
        }
    }

    pub fn recreate_swapchain(&mut self) -> bool {
        mem::replace(&mut self.recreate_swapchain, false)
    }

    pub fn done(&mut self) -> bool {
        mem::replace(&mut self.done, false)
    }
}

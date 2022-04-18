use std::collections::HashSet;

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseScrollDelta};

pub use winit::event::MouseButton as Button;
pub use winit::event::VirtualKeyCode as Key;

pub struct InputHandler {
    pressed: HashSet<Key>,
    mouse_pressed: HashSet<Button>,
    mouse_pos: (f64, f64),
    mouse_scroll: (f64, f64),
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::default(),
            mouse_pressed: HashSet::default(),
            mouse_pos: (0.0, 0.0),
            mouse_scroll: (0.0, 0.0),
        }
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.pressed.contains(&key)
    }

    #[allow(dead_code)]
    pub fn is_mouse_down(&self, button: Button) -> bool {
        self.mouse_pressed.contains(&button)
    }

    #[allow(dead_code)]
    pub fn get_mouse_pos(&self) -> (f64, f64) {
        self.mouse_pos
    }

    #[allow(dead_code)]
    pub fn get_mouse_scroll(&self) -> (f64, f64) {
        self.mouse_scroll
    }

    pub fn update_key(&mut self, key: Key, state: ElementState) {
        match state {
            ElementState::Pressed => self.pressed.insert(key),
            ElementState::Released => self.pressed.remove(&key),
        };
    }

    pub fn update_button(&mut self, button: Button, state: ElementState) {
        match state {
            ElementState::Pressed => self.mouse_pressed.insert(button),
            ElementState::Released => self.mouse_pressed.remove(&button),
        };
    }

    pub fn update_wheel(&mut self, delta: MouseScrollDelta) {
        self.mouse_scroll = match delta {
            MouseScrollDelta::PixelDelta(d) => (d.x, d.y),
            MouseScrollDelta::LineDelta(lx, ly) => (lx as f64, ly as f64),
        };
    }

    pub fn update_cursor(&mut self, position: PhysicalPosition<f64>) {
        self.mouse_pos = (position.x, position.y);
    }
}

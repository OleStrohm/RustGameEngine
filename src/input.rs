use std::collections::HashSet;

use either::Either;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseScrollDelta};

pub use winit::event::MouseButton as Button;
pub use winit::event::VirtualKeyCode as Key;

pub struct InputHandler {
    keys_pressed: HashSet<Key>,
    keys_clicked: HashSet<Key>,
    mouse_pressed: HashSet<Button>,
    mouse_clicked: HashSet<Button>,
    mouse_pos: (f64, f64),
    mouse_scroll: (f64, f64),
}

pub trait Pressable {
    fn as_button_or_key(self) -> Either<Key, Button>;
}

impl Pressable for Key {
    fn as_button_or_key(self) -> Either<Key, Button> {
        Either::Left(self)
    }
}

impl Pressable for Button {
    fn as_button_or_key(self) -> Either<Key, Button> {
        Either::Right(self)
    }
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::default(),
            keys_clicked: HashSet::default(),
            mouse_pressed: HashSet::default(),
            mouse_clicked: HashSet::default(),
            mouse_pos: (0.0, 0.0),
            mouse_scroll: (0.0, 0.0),
        }
    }

    pub fn frame(&mut self) {
        self.keys_clicked.clear();
        self.mouse_clicked.clear();
    }

    #[allow(dead_code)]
    pub fn clicked(&self, pressable: impl Pressable) -> bool {
        match pressable.as_button_or_key() {
            Either::Left(key) => self.keys_clicked.contains(&key),
            Either::Right(button) => self.mouse_clicked.contains(&button),
        }
    }

    #[allow(dead_code)]
    pub fn down(&self, pressable: impl Pressable) -> bool {
        match pressable.as_button_or_key() {
            Either::Left(key) => self.keys_pressed.contains(&key),
            Either::Right(button) => self.mouse_pressed.contains(&button),
        }
    }

    #[allow(dead_code)]
    pub fn up(&self, pressable: impl Pressable) -> bool {
        match pressable.as_button_or_key() {
            Either::Left(key) => !self.keys_pressed.contains(&key),
            Either::Right(button) => !self.mouse_pressed.contains(&button),
        }
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
            ElementState::Pressed => {
                self.keys_pressed.insert(key);
                self.keys_clicked.insert(key)
            }
            ElementState::Released => self.keys_pressed.remove(&key),
        };
    }

    pub fn update_button(&mut self, button: Button, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.mouse_pressed.insert(button);
                self.mouse_clicked.insert(button)
            }
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

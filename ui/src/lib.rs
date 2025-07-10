pub use winit;
pub type LoopType = winit::event_loop::EventLoop<()>;

use crate::InputState::{Pressed, Released};
use math::glm;
use winit::{
    event::{DeviceEvent, MouseButton, WindowEvent},
    keyboard::{Key, NamedKey},
};

const MAX_MOUSE_BUTTONS: usize = 256;
const MAX_KEY_BUTTONS: usize = 512;

pub type Sensitivity = f32;

pub enum Gesture {
    NoGesture,
    KeyHold(Key),
    KeyTrigger(Key),
    ButtonHold(MouseButton),
    ButtonTrigger(MouseButton),
    AnyOf(Vec<Gesture>),
    AllOf(Vec<Gesture>),
    QuitTrigger,
}

pub enum Analog2d {
    NoAnalog2d,

    Mouse {
        sensitivity: Sensitivity,
    },

    MouseWheel {
        sensitivity: Sensitivity,
    },

    Gestures {
        x_positive: Gesture,
        x_negative: Gesture,
        y_positive: Gesture,
        y_negative: Gesture,
        step: Sensitivity,
    },

    Sum {
        analogs: Vec<Analog2d>,
    },
}

pub struct Input {
    mouse_buttons: [InputState; MAX_MOUSE_BUTTONS],
    key_buttons: [InputState; MAX_KEY_BUTTONS],
    mouse_wheel_dir: glm::Vec2,
    mouse_rel: glm::Vec2,
    quit_requested_index: u64,
    current_index: u64,
    mouse_grabbed: bool,
    new_mouse_grabbed: bool,
}

impl Input {
    pub fn poll_gesture(&self, gesture: &Gesture) -> bool {
        match gesture {
            Gesture::QuitTrigger => self.quit_requested_index == self.current_index,
            Gesture::KeyHold(key) => match self.key_buttons[logical_key_index(key)] {
                InputState::Pressed(_) => true,
                InputState::Released(_) => false,
            },
            Gesture::KeyTrigger(key) => match self.key_buttons[logical_key_index(key)] {
                InputState::Pressed(index) => self.current_index == index,
                InputState::Released(_) => false,
            },
            Gesture::ButtonHold(button) => match self.mouse_buttons[mouse_button_index(&button)] {
                InputState::Pressed(_) => true,
                InputState::Released(_) => false,
            },
            Gesture::ButtonTrigger(button) => {
                match self.mouse_buttons[mouse_button_index(&button)] {
                    InputState::Pressed(index) => self.current_index == index,
                    InputState::Released(_) => false,
                }
            }
            Gesture::AnyOf(ref subgestures) => subgestures
                .iter()
                .any(|subgesture| self.poll_gesture(subgesture)),
            Gesture::AllOf(ref subgestures) => subgestures
                .iter()
                .all(|subgesture| self.poll_gesture(subgesture)),
            Gesture::NoGesture => false,
        }
    }

    pub fn poll_analog2d(&self, motion: &Analog2d) -> glm::Vec2 {
        match *motion {
            Analog2d::Sum { ref analogs } => analogs
                .iter()
                .map(|analog| self.poll_analog2d(analog))
                .fold(glm::vec2(0., 0.), |v1, v2| v1 + v2),
            Analog2d::Mouse { sensitivity } => (&self.mouse_rel) * sensitivity,
            Analog2d::Gestures {
                ref x_positive,
                ref x_negative,
                ref y_positive,
                ref y_negative,
                step,
            } => glm::vec2(
                if self.poll_gesture(x_positive) {
                    step
                } else if self.poll_gesture(x_negative) {
                    -step
                } else {
                    0.0
                },
                if self.poll_gesture(y_positive) {
                    step
                } else if self.poll_gesture(y_negative) {
                    -step
                } else {
                    0.0
                },
            ),
            Analog2d::MouseWheel { sensitivity } => (&self.mouse_wheel_dir) * sensitivity,
            Analog2d::NoAnalog2d => glm::vec2(0., 0.),
        }
    }

    pub fn set_cursor_grabbed(&mut self, grabbed: bool) {
        self.new_mouse_grabbed = grabbed
    }

    pub fn is_mouse_grabbed(&self) -> bool {
        self.mouse_grabbed
    }

    pub fn update_window_event(&mut self, event: &WindowEvent) {
        match event {
            winit::event::WindowEvent::KeyboardInput { event, .. } => match event.state {
                winit::event::ElementState::Pressed => {
                    self.key_buttons[logical_key_index(&event.logical_key)] =
                        Pressed(self.current_index)
                }
                winit::event::ElementState::Released => {
                    self.key_buttons[logical_key_index(&event.logical_key)] =
                        Released(self.current_index)
                }
            },
            winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    self.mouse_wheel_dir += glm::vec2(*x, *y);
                }
                _ => {}
            },
            winit::event::WindowEvent::MouseInput { state, button, .. } => match state {
                winit::event::ElementState::Pressed => {
                    self.mouse_buttons[mouse_button_index(button) as usize] =
                        Pressed(self.current_index)
                }
                winit::event::ElementState::Released => {
                    self.mouse_buttons[mouse_button_index(button) as usize] =
                        Released(self.current_index)
                }
            },
            winit::event::WindowEvent::CloseRequested => {
                self.quit_requested_index = self.current_index
            }
            _ => {}
        }
    }

    pub fn update_device_event(&mut self, event: &DeviceEvent) {
        match event {
            winit::event::DeviceEvent::Motion { axis, value } => {
                if *axis < 2 {
                    self.mouse_rel[*axis as usize] += *value as f32;
                }
            }
            _ => {}
        }
    }

    pub fn create() -> Self {
        Self {
            mouse_buttons: [InputState::Released(0); MAX_MOUSE_BUTTONS],
            key_buttons: [InputState::Released(0); MAX_KEY_BUTTONS],
            mouse_wheel_dir: glm::vec2(0.0, 0.0),
            mouse_rel: glm::vec2(0.0, 0.0),
            quit_requested_index: 0,
            current_index: 1,
            mouse_grabbed: false,
            new_mouse_grabbed: false,
        }
    }

    pub fn tick_reset(&mut self) {
        self.current_index += 1;
        self.mouse_rel = glm::vec2(0.0, 0.0);
        self.mouse_wheel_dir = glm::vec2(0.0, 0.0);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum InputState {
    Pressed(u64),
    Released(u64),
}

fn mouse_button_index(button: &MouseButton) -> usize {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
        MouseButton::Other(id) => *id as usize,
        MouseButton::Back => 3,
        MouseButton::Forward => 4,
    }
}

// useful when parameting programmatically or text config. But physical might be more suited when parameting with an interface detecting keys.
fn logical_key_index(key: &Key) -> usize {
    match key {
        // Named keys (function keys, arrows, etc.)
        Key::Named(named_key) => match named_key {
            NamedKey::Escape => 0,
            NamedKey::Tab => 1,
            NamedKey::Backspace => 2,
            NamedKey::Enter => 3,
            NamedKey::Space => 4,
            NamedKey::Insert => 5,
            NamedKey::Delete => 6,
            NamedKey::Home => 7,
            NamedKey::End => 8,
            NamedKey::PageUp => 9,
            NamedKey::PageDown => 10,
            NamedKey::ArrowLeft => 11,
            NamedKey::ArrowUp => 12,
            NamedKey::ArrowRight => 13,
            NamedKey::ArrowDown => 14,
            NamedKey::F1 => 15,
            NamedKey::F2 => 16,
            NamedKey::F3 => 17,
            NamedKey::F4 => 18,
            NamedKey::F5 => 19,
            NamedKey::F6 => 20,
            NamedKey::F7 => 21,
            NamedKey::F8 => 22,
            NamedKey::F9 => 23,
            NamedKey::F10 => 24,
            NamedKey::F11 => 25,
            NamedKey::F12 => 26,
            NamedKey::Shift => 27,
            NamedKey::Control => 28,
            NamedKey::Alt => 29,
            NamedKey::Super => 30,
            NamedKey::CapsLock => 31,
            NamedKey::NumLock => 32,
            NamedKey::ScrollLock => 33,
            _ => 511, // Fallback for unmapped named keys
        },
        // Character keys (letters, numbers, symbols)
        Key::Character(ch) => {
            let ch_lower = ch
                .chars()
                .next()
                .unwrap_or('\0')
                .to_lowercase()
                .next()
                .unwrap_or('\0');
            match ch_lower {
                // Letters a-z
                'a'..='z' => 34 + (ch_lower as usize - 'a' as usize),
                // Numbers 0-9
                '0'..='9' => 60 + (ch_lower as usize - '0' as usize),
                // Common symbols
                '`' => 70,
                '-' => 71,
                '=' => 72,
                '[' => 73,
                ']' => 74,
                '\\' => 75,
                ';' => 76,
                '\'' => 77,
                ',' => 78,
                '.' => 79,
                '/' => 80,
                // Shifted symbols
                '~' => 81,
                '!' => 82,
                '@' => 83,
                '#' => 84,
                '$' => 85,
                '%' => 86,
                '^' => 87,
                '&' => 88,
                '*' => 89,
                '(' => 90,
                ')' => 91,
                '_' => 92,
                '+' => 93,
                '{' => 94,
                '}' => 95,
                '|' => 96,
                ':' => 97,
                '"' => 98,
                '<' => 99,
                '>' => 100,
                '?' => 101,
                ' ' => 4, // Space (same as Named(NamedKey::Space))
                _ => 510, // Fallback for unmapped characters
            }
        }
        // Unidentified keys
        Key::Unidentified(_) => 509,
        // Dead keys (for composition)
        Key::Dead(_) => 508,
    }
}

pub struct Binding {
    pub exit: Gesture,
    pub movement: Analog2d,
    pub look: Analog2d,
    pub jump: Gesture,
    pub fall: Gesture,
    pub fullscreen: Gesture,
    pub scroll: Analog2d,
    pub toggle_mouse: Gesture,
    pub toggle_torch_light: Gesture,
    pub speedup: Gesture,
    pub speeddown: Gesture,
}

impl Binding {
    pub fn create() -> Self {
        Self {
            exit: Gesture::KeyTrigger(Key::Named(NamedKey::Escape)),
            movement: Analog2d::Gestures {
                x_positive: Gesture::AnyOf(vec![
                    Gesture::KeyHold(Key::Character("d".into())),
                    Gesture::KeyHold(Key::Named(NamedKey::ArrowRight)),
                ]),
                x_negative: Gesture::AnyOf(vec![
                    Gesture::KeyHold(Key::Character("q".into())),
                    Gesture::KeyHold(Key::Named(NamedKey::ArrowLeft)),
                ]),
                y_positive: Gesture::AnyOf(vec![
                    Gesture::KeyHold(Key::Character("z".into())),
                    Gesture::KeyHold(Key::Named(NamedKey::ArrowUp)),
                ]),
                y_negative: Gesture::AnyOf(vec![
                    Gesture::KeyHold(Key::Character("s".into())),
                    Gesture::KeyHold(Key::Named(NamedKey::ArrowDown)),
                ]),
                step: 0.15,
            },
            look: Analog2d::Mouse { sensitivity: 0.015 },
            jump: Gesture::KeyHold(Key::Named(NamedKey::Space)),
            fall: Gesture::KeyHold(Key::Character("c".into())),
            fullscreen: Gesture::KeyTrigger(Key::Character("f".into())),
            scroll: Analog2d::MouseWheel { sensitivity: 0.030 },
            toggle_mouse: Gesture::ButtonHold(MouseButton::Right),
            toggle_torch_light: Gesture::KeyTrigger(Key::Character("t".into())),
            speedup: Gesture::KeyTrigger(Key::Character("+".into())),
            speeddown: Gesture::KeyTrigger(Key::Character("-".into())),
        }
    }
}

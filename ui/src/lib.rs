pub use winit;
pub type LoopType = winit::event_loop::EventLoop<()>;

use winit::event::{VirtualKeyCode, MouseButton, Event};
use math::glm;
use crate::InputState::{Pressed, Released};

const MAX_MOUSE_BUTTONS: usize = 256;
const MAX_KEY_BUTTONS: usize = 512;

pub type Sensitivity = f32;

pub enum Gesture {
    NoGesture,
    KeyHold(VirtualKeyCode),
    KeyTrigger(VirtualKeyCode),
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
        match *gesture {
            Gesture::QuitTrigger => self.quit_requested_index == self.current_index,
            Gesture::KeyHold(code) => match self.key_buttons[code as usize] {
                InputState::Pressed(_) => true,
                InputState::Released(_) => false,
            },
            Gesture::KeyTrigger(code) => match self.key_buttons[code as usize] {
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
            Analog2d::MouseWheel { sensitivity } => { (&self.mouse_wheel_dir) * sensitivity }
            Analog2d::NoAnalog2d => glm::vec2(0., 0.),
        }
    }

    pub fn set_cursor_grabbed(&mut self, grabbed: bool) {
        self.new_mouse_grabbed = grabbed
    }

    pub fn is_mouse_grabbed(&self) -> bool {
        self.mouse_grabbed
    }

    pub fn update(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                } => match (input.virtual_keycode, input.state) {
                    (Some(key), winit::event::ElementState::Pressed) => {
                        self.key_buttons[key as usize] = Pressed(self.current_index)
                    }
                    (Some(key), winit::event::ElementState::Released) => {
                        self.key_buttons[key as usize] = Released(self.current_index)
                    }
                    (None, _) => {}
                },
                winit::event::WindowEvent::Focused(flag) => {}
                winit::event::WindowEvent::MouseWheel {
                    device_id,
                    delta,
                    phase,
                    ..
                } => match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        self.mouse_wheel_dir += glm::vec2(*x, *y);
                    }
                    _ => {}
                },
                winit::event::WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                    ..
                } => match state {
                    winit::event::ElementState::Pressed => {
                        self.mouse_buttons[mouse_button_index(button) as usize] =
                            Pressed(self.current_index)
                    }
                    winit::event::ElementState::Released => {
                        self.mouse_buttons[mouse_button_index(button) as usize] =
                            Released(self.current_index)
                    }
                },
                winit::event::WindowEvent::CursorMoved {
                    device_id,
                    position,
                    ..
                } => {}
                winit::event::WindowEvent::CloseRequested => {
                    self.quit_requested_index = self.current_index
                }
                _ => {}
            },

            winit::event::Event::DeviceEvent {
                event: winit::event::DeviceEvent::Motion { axis, value },
                ..
            } => {
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
    }
}


type Key = VirtualKeyCode;
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
    pub speeddown: Gesture
}

impl Binding {
    pub fn create() -> Self {
        Self {
            exit: Gesture::KeyTrigger(Key::Escape),
            movement: Analog2d::Gestures {
                x_positive: Gesture::AnyOf(vec![
                    Gesture::KeyHold(Key::D),
                    Gesture::KeyHold(Key::Right),
                ]),
                x_negative: Gesture::AnyOf(vec![
                    Gesture::KeyHold(Key::Q),
                    Gesture::KeyHold(Key::Left),
                ]),
                y_positive: Gesture::AnyOf(vec![
                    Gesture::KeyHold(Key::Z),
                    Gesture::KeyHold(Key::Up),
                ]),
                y_negative: Gesture::AnyOf(vec![
                    Gesture::KeyHold(Key::S),
                    Gesture::KeyHold(Key::Down),
                ]),
                step: 0.15,
            },
            look: Analog2d::Mouse { sensitivity: 0.015 },
            jump: Gesture::KeyHold(Key::Space),
            fall: Gesture::KeyHold(Key::C),
            fullscreen: Gesture::KeyTrigger(Key::F),
            scroll: Analog2d::MouseWheel { sensitivity: 0.030 },
            toggle_mouse: Gesture::ButtonHold(MouseButton::Right),
            toggle_torch_light: Gesture::KeyTrigger(Key::T),
            speedup: Gesture::KeyTrigger(Key::NumpadAdd),
            speeddown: Gesture::KeyTrigger(Key::NumpadSubtract)
        }
    }
}

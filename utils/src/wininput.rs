use winit::event::{
    DeviceEvent, ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
};
use nalgebra::Vector2;

pub enum StateChange {
    Keyboard,
    MouseButton,
    MouseScroll,
    MouseMotion,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DoublePressState {
    Released,
    PressedOnce(f32),
    ReleasedAfterPressed(f32),
    Pressed,
}

const CST_MAX_NUMBER_STATE_CHANGE: usize = 4;
const CST_MAX_NUMBER_KEY: usize = u8::max_value() as usize;
const CST_MAX_NUMBER_BUTTON: usize = u8::max_value() as usize;

const MAX_TIME_DOUBLE_PRESS: f32 = 0.4;

type StateChangeArray = [bool; CST_MAX_NUMBER_STATE_CHANGE];
type KeyStateArray = [ElementState; CST_MAX_NUMBER_KEY];
type KeyDoublePressArray = [DoublePressState; CST_MAX_NUMBER_KEY];
type ButtonStateArray = [ElementState; CST_MAX_NUMBER_BUTTON];

pub struct WinInput {
    states_changes: StateChangeArray,

    mouse_scroll: f32,
    mouse_scroll_sensitivity: f32,

    mouse_motion_offset: Vector2<f32>,
    mouse_motion_sensitivity: f32,

    keys_states: KeyStateArray,
    double_pressed_states: KeyDoublePressArray,
    buttons_states: ButtonStateArray,

    time: f32,
}

impl WinInput {
    fn set_state_updated(&mut self, state: StateChange) {
        self.states_changes[state as usize] = true;
    }

    pub fn update_time(&mut self, dt: f32) {
        self.time += dt;
    }

    pub fn updated(&mut self, state: StateChange) -> bool {
        let k = state as usize;
        if self.states_changes[k] {
            self.states_changes[k] = false;
            true
        } else {
            false
        }
    }

    fn handle_mouse_wheel(&mut self, input: MouseScrollDelta) {
        match input {
            MouseScrollDelta::LineDelta(dx, dy) => {
                self.mouse_scroll += self.mouse_scroll_sensitivity * (dx + dy);
                self.mouse_scroll = self.mouse_scroll.max(0.).min(1.);

                // FIXME: should update state only if scroll change from previous
                self.set_state_updated(StateChange::MouseScroll);
                // FIXME-END
            }
            _ => (),
        }
    }

    fn handle_mouse_motion(&mut self, dx: f32, dy: f32) {
        self.mouse_motion_offset = Vector2::new(dx, -dy) * self.mouse_motion_sensitivity;
        self.set_state_updated(StateChange::MouseMotion);
    }

    pub fn get_scroll(&self) -> f32 {
        self.mouse_scroll
    }

    pub fn get_mouse_offset(&self) -> Vector2<f32> {
        self.mouse_motion_offset
    }

    pub fn is_pressed(&self, k: VirtualKeyCode) -> bool {
        self.keys_states[k as usize] == ElementState::Pressed
    }

    pub fn is_pressed_once(&mut self, k: VirtualKeyCode) -> bool {
        let k = k as usize;
        if self.keys_states[k] == ElementState::Pressed {
            self.keys_states[k] = ElementState::Released;
            return true;
        }
        return false;
    }

    pub fn is_double_pressed(&mut self, k: VirtualKeyCode) -> bool {
        let k = k as usize;

        if self.double_pressed_states[k] == DoublePressState::Pressed {
            self.double_pressed_states[k] = DoublePressState::Released;
            return true;
        }

        return false;
    }

    fn button_id(&self, button: MouseButton) -> usize {
        match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Other(v) => v as usize,
        }
    }

    pub fn is_button_pressed(&self, k: MouseButton) -> bool {
        self.buttons_states[self.button_id(k)] == ElementState::Pressed
    }

    pub fn on_device_event(&mut self, input: DeviceEvent) {
        match input {
            DeviceEvent::MouseWheel { delta } => self.handle_mouse_wheel(delta),
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                self.handle_mouse_motion(dx as f32, dy as f32)
            }
            _ => (),
        }
    }

    pub fn on_mouse_input(&mut self, input: MouseButton, state: ElementState) {
        let k = self.button_id(input);

        if state != self.buttons_states[k] {
            self.buttons_states[k] = state;
            self.set_state_updated(StateChange::MouseButton);
        }
    }

    pub fn on_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(k) = input.virtual_keycode {
            let k = k as usize;

            if input.state == ElementState::Pressed {
                self.double_pressed_states[k] = match self.double_pressed_states[k] {
                    DoublePressState::Released => DoublePressState::PressedOnce(self.time),
                    DoublePressState::ReleasedAfterPressed(t)
                        if self.time - t < MAX_TIME_DOUBLE_PRESS =>
                    {
                        DoublePressState::Pressed
                    }
                    _ => DoublePressState::Released,
                }
            } else {
                self.double_pressed_states[k] = match self.double_pressed_states[k] {
                    DoublePressState::PressedOnce(t) if self.time - t < MAX_TIME_DOUBLE_PRESS => {
                        DoublePressState::ReleasedAfterPressed(self.time)
                    }
                    _ => DoublePressState::Released,
                };
            }

            if input.state != self.keys_states[k] {
                self.keys_states[k] = input.state;
                self.set_state_updated(StateChange::Keyboard);
            }
        }
    }

    pub fn new(mouse_scroll_sensitivity: f32, mouse_motion_sensitivity: f32) -> Self {
        WinInput {
            mouse_scroll: 0.5,
            mouse_scroll_sensitivity,

            mouse_motion_offset: Vector2::zeros(),
            mouse_motion_sensitivity,

            keys_states: [ElementState::Released; CST_MAX_NUMBER_KEY],
            double_pressed_states: [DoublePressState::Released; CST_MAX_NUMBER_KEY],
            buttons_states: [ElementState::Released; CST_MAX_NUMBER_BUTTON],
            states_changes: [false; CST_MAX_NUMBER_STATE_CHANGE],

            time: 0.0,
        }
    }
}

impl Default for WinInput {
    fn default() -> Self {
        WinInput::new(0.005, 1.)
    }
}

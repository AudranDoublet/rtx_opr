use glutin::event::{DeviceEvent, ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode};
use nalgebra::Vector2;

pub enum StateChange {
    Keyboard,
    MouseScroll,
    MouseMotion,
}

const CST_MAX_NUMBER_STATE_CHANGE: usize = 3;
const CST_MAX_NUMBER_KEY: usize = u8::max_value() as usize;

type StateChangeArray = [bool; CST_MAX_NUMBER_STATE_CHANGE];
type KeyStateArray = [ElementState; CST_MAX_NUMBER_KEY];

pub struct WinInput {
    states_changes: StateChangeArray,
    scroll_intensity: f32,
    scroll: f32,
    last_mouse_coords: Option<Vector2<f32>>,
    keys_states: KeyStateArray,
}

impl WinInput {
    fn set_state_updated(&mut self, state: StateChange) {
        self.states_changes[state as usize] = true;
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
                self.scroll += self.scroll_intensity * (dx + dy);
                self.scroll = self.scroll.clamp(0., 1.);

                // FIXME: should update state only if scroll change from previous
                self.set_state_updated(StateChange::MouseScroll);
                // FIXME-END
            }
            _ => (),
        }
    }

    fn handle_mouse_motion(&mut self, dx: f64, dy: f64) {}

    pub fn get_scroll(&self) -> f32 {
        self.scroll
    }

    pub fn is_pressed(&self, k: VirtualKeyCode) -> bool {
        self.keys_states[k as usize] == ElementState::Pressed
    }

    pub fn on_device_event(&mut self, input: DeviceEvent) {
        match input {
            DeviceEvent::MouseWheel { delta } => self.handle_mouse_wheel(delta),
            DeviceEvent::MouseMotion { delta: (dx, dy) } => self.handle_mouse_motion(dx, dy),
            _ => (),
        }
    }

    pub fn on_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(k) = input.virtual_keycode {
            let k = k as usize;
            if input.state != self.keys_states[k] {
                self.keys_states[k] = input.state;
                self.set_state_updated(StateChange::Keyboard);
            }
        }
    }

    pub fn new(scroll: f32, scroll_intensity: f32) -> Self {
        WinInput {
            scroll,
            scroll_intensity,
            last_mouse_coords: None,
            keys_states: [ElementState::Released; CST_MAX_NUMBER_KEY],
            states_changes: [false; CST_MAX_NUMBER_STATE_CHANGE],
        }
    }
}

impl Default for WinInput {
    fn default() -> Self {
        WinInput::new(0.5, 0.005)
    }
}

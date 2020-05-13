use glutin::event::{ElementState, KeyboardInput, VirtualKeyCode};

const CST_MAX_NUMBER_KEY: usize = u8::max_value() as usize;

type KeyStateArray = [ElementState; CST_MAX_NUMBER_KEY];

pub struct WinInput {
    states: KeyStateArray,
}

#[inline]
fn hash(k: VirtualKeyCode) -> usize {
    k as usize
}

impl WinInput {
    pub fn is_pressed(&self, k: VirtualKeyCode) -> bool {
        self.states[hash(k)] == ElementState::Pressed
    }

    pub fn update(&mut self, input: KeyboardInput) {
        if let Some(k) = input.virtual_keycode {
            self.states[hash(k)] = input.state;
        }
    }

    pub fn new() -> Self {
        WinInput {
            states: [ElementState::Released; CST_MAX_NUMBER_KEY],
        }
    }
}

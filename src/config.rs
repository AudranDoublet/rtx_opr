use winit::event::VirtualKeyCode as KeyCode;

#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub resolution: [u32; 2],
    pub msaa: u32,
    pub vsync: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resolution: [1920, 1080],
            msaa: 1,
            vsync: true,
        }
    }
}

pub enum Layout {
    Azerty,
    Qwerty,
}

impl Layout {
    pub fn parse(name: &str) -> Layout {
        match name {
            "azerty" | "fr" => Layout::Azerty,
            "qwerty" | "us" | "uk" | "en" => Layout::Qwerty,
            _ => panic!("unknown layout"),
        }
    }

    pub fn forward(&self) -> KeyCode {
        match self {
            Layout::Azerty => KeyCode::Z,
            Layout::Qwerty => KeyCode::W,
        }
    }

    pub fn backward(&self) -> KeyCode {
        match self {
            Layout::Azerty => KeyCode::S,
            Layout::Qwerty => KeyCode::S,
        }
    }

    pub fn right(&self) -> KeyCode {
        match self {
            Layout::Azerty => KeyCode::Q,
            Layout::Qwerty => KeyCode::A,
        }
    }

    pub fn left(&self) -> KeyCode {
        match self {
            Layout::Azerty => KeyCode::D,
            Layout::Qwerty => KeyCode::D,
        }
    }
}

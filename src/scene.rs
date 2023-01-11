use web_sys::Window;

use crate::keyboard::Keyboard;

pub struct Scene {
    pub keyboard: Keyboard,
    pub time: f32,
    pub player_center: (f32, f32),
    pub player: (f32, f32),
    pub speed: f32,
}

impl Scene {
    pub fn new(window: &Window) -> Self {
        let keyboard = Keyboard::new(window);

        Self {
            keyboard,
            time: 0.0,
            player_center: (0.0, 0.0),
            player: (0.0, 0.0),
            speed: 5.0
        }
    }
}
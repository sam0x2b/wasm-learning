use web_sys::Window;

use crate::keyboard::Keyboard;

pub struct Scene {
    pub keyboard: Keyboard,
    pub time: f32,
    pub player_center: (f32, f32),
    pub player: (f32, f32),
    pub speed: f32,

    pub line: [f32; 5*2],
}

impl Scene {
    pub fn new(window: &Window) -> Self {
        let keyboard = Keyboard::new(window);
        let line = [
            1.0, 1.0,
            2.0, -1.0,
            3.0, 2.0,
            4.0, -2.0,
            5.0, 3.0,
        ];

        Self {
            keyboard,
            time: 0.0,
            player_center: (0.0, 0.0),
            player: (0.0, 0.0),
            speed: 5.0,
            line
        }
    }
}
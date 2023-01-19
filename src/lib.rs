mod keyboard;
mod renderer;
mod vec2;

use keyboard::Key;
use keyboard::Keyboard;
use renderer::Renderer;
use vec2::Vec2;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
// use web_sys::{ Document, Window };
use web_sys::HtmlCanvasElement;
use web_sys::WebGl2RenderingContext as GL;

struct Sprite {
    position: Vec2,
    velocity: Vec2,
}

impl Sprite {
    fn new() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 0.0),
        }
    }
}

struct Player {
    can_jump: bool,
    position: Vec2, // similar to sprite. theres probably a better way to do this, but i'll think about it later
    velocity: Vec2,
}

impl Player {
    fn new() -> Self {
        Self {
            can_jump: true,
            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 0.0),
        }
    }
}

struct Scene {
    player: Player
}

impl Scene {
    fn new() -> Self {
        Self {
            player: Player::new()
        }
    }
}

#[wasm_bindgen]
pub struct Client {
    // window: Window,
    canvas: HtmlCanvasElement,
    keyboard: Keyboard,

    renderer: Renderer, // Holds GL related state
    scene: Scene
}

#[wasm_bindgen]
impl Client {
    /// Create a new web client
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        let window = web_sys::window().expect("Should've obtained window");
        let document = window.document().expect("Should've obtained document");
        let canvas = document
            .get_element_by_id("canvas")
            .expect("Should've found element #canvas")
            .dyn_into::<HtmlCanvasElement>()
            .expect("#canvas should've been cast to HtmlCanvasElement");

        Self {
            renderer: Renderer::new(&canvas),
            scene: Scene::new(),
            keyboard: Keyboard::new(&window),
            // window,
            canvas,
        }
    }

    // called once every `requestAnimationFrame`
    #[no_mangle]
    pub fn update(&mut self, dt: f32) {
        let player = &mut self.scene.player;
        let keyboard = &self.keyboard;

        // controls
        const SPEED: f32 = 15.0;

        let mut d_velocity_x = 0.0;
        if keyboard.is_down(Key::Left) { d_velocity_x += -SPEED; }
        if keyboard.is_down(Key::Right) { d_velocity_x += SPEED; }
        player.velocity.x = d_velocity_x;

        if keyboard.is_down(Key::Up) {
            if player.can_jump {
                player.can_jump = false;
                player.velocity.y = 100.0;
            }
        } else {
            player.can_jump = true;
        }

        // gravity
        player.velocity.y -= 3.0;

        // update position based on velocity
        player.position += player.velocity * (dt / 60.0);
    }

    // called once every `requestAnimationFrame`
    #[no_mangle]
    pub fn render(&mut self) {
        let renderer = &self.renderer;
        let scene = &self.scene;
        let gl = &renderer.gl;

        // Check canvas size.
        let c = &self.canvas;
        let size        = (c.width() as i32, c.height() as i32);
        let client_size = (c.client_width(), c.client_height());

        if size != client_size {
            // let test = (c.client_width(), c.client_height());
            // web_sys::console::log_1(&format!("size: {:#?}\nclient_size: {:#?}\ntest: {:#?}",
            //     size,
            //     client_size,
            //     test
            // ).into());

            c.set_width(client_size.0 as u32);
            c.set_height(client_size.1 as u32);

            gl.viewport(0, 0, client_size.0, client_size.1);

            gl.use_program(Some(&renderer.program));
            gl.uniform2f(Some(&renderer.u_canvas_size), client_size.0 as f32, client_size.1 as f32);
            gl.use_program(None);
        }

        // Update uniforms
        gl.use_program(Some(&renderer.program));
        gl.uniform2f(Some(&renderer.u_displacement), scene.player.position.x, scene.player.position.y);

        // Bind
        gl.bind_texture(GL::TEXTURE_2D, Some(&renderer.texture));
        gl.bind_vertex_array(Some(&renderer.vao));

        // Draw
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT);
        gl.draw_arrays(GL::TRIANGLES, 0, renderer.vertex_count);

        // Unbind
        gl.bind_vertex_array(None);
        gl.bind_texture(GL::TEXTURE_2D, None);
        gl.use_program(None);
    }
}
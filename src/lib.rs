use renderer::Renderer;
use scene::Scene;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
// use web_sys::{ Document, Window };
use web_sys::HtmlCanvasElement;
use web_sys::WebGl2RenderingContext as GL;

use crate::keyboard::Key;

mod scene;
mod keyboard;
mod renderer;

#[wasm_bindgen]
pub struct Client {
    // window: Window,
    canvas: HtmlCanvasElement,

    renderer: Renderer, // Holds GL related state
    scene: Scene, // Holds scene state
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
            scene: Scene::new(&window),

            // window,
            canvas,
        }
    }

    // called once every `requestAnimationFrame`
    #[no_mangle]
    pub fn update(&mut self, mut dt: f32) {
        let scene = &mut self.scene;
        let keyboard = &scene.keyboard;

        // Speed Controls
        let player_center = &mut scene.player_center;
        if keyboard.is_down(Key::Turbo) { // Accelerate
            scene.speed += 2.0;
        } else {
            scene.speed = 5.0;
        }
        if keyboard.is_down(Key::Right) { player_center.0 += scene.speed; }
        if keyboard.is_down(Key::Left) { player_center.0 += -scene.speed; }

        // Wiggle Mechanics
        if keyboard.is_down(Key::Up) { dt *= 2.0; }
        if keyboard.is_down(Key::Down) { dt *= 0.5; }
        scene.time += dt;

        let i = (scene.time * 0.001).cos() * 4.0 + 2.0; // intensity
        let offset = (
            i * (scene.time * 0.006).cos() * 12.0,
            i * (scene.time * 0.01).sin() * 12.0
        );

        // Final Player Position
        scene.player = (
            player_center.0 + offset.0,
            player_center.1 + offset.1,
        )
    }

    // called once every `requestAnimationFrame`
    #[no_mangle]
    pub fn render(&mut self) {
        let renderer = &self.renderer;
        let gl = &renderer.gl;

        // Check canvas size.
        let c = &self.canvas;
        let size        = (c.width() as i32, c.height() as i32);
        let client_size = (c.client_width(), c.client_height());

        if size != client_size {
            c.set_height(c.client_height() as u32);
            c.set_width(c.client_width() as u32);

            gl.viewport(0, 0, c.client_width(), c.client_height());

            gl.use_program(Some(&renderer.program));
            gl.uniform2f(Some(&renderer.u_canvas_size), c.client_width() as f32, c.client_height() as f32);
            gl.use_program(None);
        }

        // Update uniforms
        gl.use_program(Some(&renderer.program));
        let scene = &self.scene;
        gl.uniform2f(Some(&renderer.u_displacement), scene.player.0, scene.player.1);

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
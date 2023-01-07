use std::borrow::Borrow;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ HtmlCanvasElement, HtmlImageElement, WebGl2RenderingContext as GL, WebGlProgram,
    WebGlShader, WebGlTexture, WebGlUniformLocation, WebGlVertexArrayObject };

struct Renderer {
    canvas: HtmlCanvasElement,
    gl: Rc<GL>,

    texture: WebGlTexture,
    vao: WebGlVertexArrayObject,
    vertex_count: i32,

    program: Rc<WebGlProgram>, // keeping it here for now, later it will be attached to a draw list
    u_displacement: WebGlUniformLocation,
    u_canvas_size: WebGlUniformLocation,
}

impl Renderer {
    fn new() -> Self {
        // WebGL context
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        let gl = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<GL>()
            .unwrap();
        let gl = Rc::new(gl);

        // Shader program
        let vert_shader = compile_shader(&gl, GL::VERTEX_SHADER, include_str!("./shader.vert")).unwrap();
        let frag_shader = compile_shader(&gl, GL::FRAGMENT_SHADER, include_str!("./shader.frag")).unwrap();
        let program = link_program(&gl, &vert_shader, &frag_shader).unwrap();
        let program = Rc::new(program);

        gl.use_program(Some(&program));

        // Uniforms
        let u_displacement = gl.get_uniform_location(program.as_ref(), "u_displacement").unwrap();
        let u_canvas_size = gl.get_uniform_location(program.as_ref(), "u_canvas_size").unwrap();

        // Texture setup (see https://webgl2fundamentals.org/webgl/lessons/webgl-image-processing.html)
        let image = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("texture")
            .unwrap()
            .dyn_into::<HtmlImageElement>()
            .unwrap(); // ((another unwrap, purely because `?;` doesn't match return value (sorry sam)

        // (create and bind texture)
        let texture = gl.create_texture().unwrap();
        gl.active_texture(GL::TEXTURE0); // fixme when we get to actually making stuff, we need to be sure we are using GL::TEXTUREx thing (coz compatibility)
        gl.bind_texture(GL::TEXTURE_2D, Some(&texture));

        // (image uniform location)
        let u_texture = gl.get_uniform_location(program.as_ref(), "u_texture").unwrap();
        gl.uniform1i(Some(&u_texture), 0); // associates the uniform `u_texture` with texture unit 0.

        gl.pixel_storei(GL::UNPACK_FLIP_Y_WEBGL, 1);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
        gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
            GL::TEXTURE_2D,
            0,
            GL::RGBA as i32,
            GL::RGBA,
            GL::UNSIGNED_BYTE,
            &image,
        )
        .expect("Not a valid texture");

        // VAO and buffer objects
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));
        let vertex_count = 6;

        setup_buffer(gl.clone(), program.clone(), "a_position", 2, &[
            // (hypothetically we could use indexing to cut this down to just /four/ vertices
            // worth of data... but right now i just dont feel like it)
            -60.0, 60.0, // top left
            -60.0, -60.0, // bottom left
            60.0, -60.0, // bottom right
            -60.0, 60.0, // top left
            60.0, -60.0, // bottom right
            60.0, 60.0, // top right
        ]).unwrap();

        setup_buffer(gl.clone(), program.clone(), "a_uv", 2, &[
            0.0, 1.0, // top left
            0.0, 0.0, // bottom left
            1.0, 0.0, // bottom right
            0.0, 1.0, // top left
            1.0, 0.0, // bottom right
            1.0, 1.0, // top right
        ]).unwrap();

        // Unbind things and return
        gl.bind_vertex_array(None);
        gl.bind_texture(GL::TEXTURE_2D, None);
        gl.use_program(None);

        Renderer {
            canvas,
            gl,

            texture,
            vao,
            vertex_count,

            program,
            u_displacement,
            u_canvas_size,
        }
    }
}

struct Scene {
    time: f32,
    player: (f32, f32),
}

impl Scene {
    fn new() -> Self {
        Self {
            time: 0.0,
            player: (0.0, 0.0),
        }
    }
}

#[wasm_bindgen]
pub struct Client {
    renderer: Renderer,
    scene: Scene,
}

#[wasm_bindgen]
impl Client {
    /// Create a new web client
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Client {
            renderer: Renderer::new(),
            scene: Scene::new(),
        }
    }

    // called once every `requestAnimationFrame`
    #[no_mangle]
    pub fn update(&mut self, dt: f32) {
        let scene = &mut self.scene;

        scene.time += dt;

        let i = (scene.time * 0.001).cos() * 4.0 + 2.0; // intensity
        scene.player = (
            i * (scene.time * 0.006).cos() * 12.0,
            i * (scene.time * 0.01).sin() * 12.0
        );
    }

    // called once every `requestAnimationFrame`
    #[no_mangle]
    pub fn render(&mut self) {
        let renderer = &self.renderer;
        let gl = &renderer.gl;

        // Check canvas size.
        let c = &renderer.canvas;
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

fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if !gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    } else {
        Ok(shader)
    }
}

fn link_program(
    gl: &GL,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if !gl
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    } else {
        Ok(program)
    }
}

// helper for setting up buffers and attributes
fn setup_buffer(
    gl: Rc<GL>,
    program: Rc<WebGlProgram>,
    attrib: &str,
    size: i32,
    data: &[f32],
) -> Result<(), JsValue> {
    // Create and bind
    let buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));

    // Upload
    unsafe {
        let array = js_sys::Float32Array::view(data); // memory danger
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &array, GL::STATIC_DRAW);
    }

    // Attribute thingies
    let attrib = gl.get_attrib_location(&program, attrib) as u32;
    gl.vertex_attrib_pointer_with_i32(attrib, size, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(attrib);

    // Unbind
    gl.bind_buffer(GL::ARRAY_BUFFER, None);

    Ok(())
}

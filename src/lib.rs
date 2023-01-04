use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as GL;
use web_sys::*; // todo selective imports later

#[wasm_bindgen]
pub struct Client {
    // todo add game state here
    time: f32,
    program: Rc<WebGlProgram>, // keeping it here for now, later it will be attached to a draw list
    gl: Rc<GL>,
}

#[wasm_bindgen]
impl Client {
    /// Create a new web client
    #[wasm_bindgen(constructor)]
    pub fn new() -> Client {
        console_error_panic_hook::set_once();

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let gl = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<GL>()
            .unwrap();
        let gl = Rc::new(gl);

        // Shader Program
        let vert_shader = compile_shader(&gl, GL::VERTEX_SHADER, include_str!("./shader.vert")).unwrap();
        let frag_shader = compile_shader(&gl, GL::FRAGMENT_SHADER, include_str!("./shader.frag")).unwrap();
        let program = link_program(&gl, &vert_shader, &frag_shader).unwrap();
        let program = Rc::new(program);
        gl.use_program(Some(&program));

        let time: f32 = 0.0;

        Client { time, gl, program }
    }

    // called once before the render loop
    #[no_mangle]
    pub fn start(&self) -> Result<(), JsValue> {
        let gl = &self.gl;

        // Texture Setup
        // (referring to https://webgl2fundamentals.org/webgl/lessons/webgl-image-processing.html)
        let image = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("texture")
            .unwrap()
            .dyn_into::<web_sys::HtmlImageElement>()?;

        // (create and bind texture)
        let texture = gl.create_texture().unwrap();
        gl.active_texture(GL::TEXTURE0); // fixme when we get to actually making stuff, we need to be sure we are using GL::TEXTUREx thing (coz compatibility)
        gl.bind_texture(GL::TEXTURE_2D, Some(&texture));

        // (image uniform location)
        let u_texture = gl.get_uniform_location(self.program.as_ref(), "u_texture").unwrap();
        gl.uniform1i(Some(&u_texture), 0); // tell the shader to look at texture unit 0

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

        Ok(())
    }

    // called before `render()` once every requestAnimationFrame
    #[no_mangle]
    pub fn update(&mut self, dt: f32) {
        // todo wobble that smug look
        self.time += dt;
    }

    // called once every `requestAnimationFrame`
    #[no_mangle]
    pub fn render(&mut self) {
        let gl = &self.gl;

        // VAO and buffer objects
        const VERTEX_COUNT: usize = 6;
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));

        let arr = [
            // (hypothetically we could use indexing to cut this down to just /four/ vertices
            // worth of data... but right now i just dont feel like it)
            -0.7, 0.7, 0.0, // top left
            -0.7, -0.7, 0.0, // bottom left
            0.7, -0.7, 0.0, // bottom right
            -0.7, 0.7, 0.0, // top left
            0.7, -0.7, 0.0, // bottom right
            0.7, 0.7, 0.0, // top right
        ];
        setup_buffer(gl.clone(), self.program.clone(), "a_position", 3, &arr).unwrap();

        let arr = [
            0.0, 1.0, // top left
            0.0, 0.0, // bottom left
            1.0, 0.0, // bottom right
            0.0, 1.0, // top left
            1.0, 0.0, // bottom right
            1.0, 1.0, // top right
        ];
        setup_buffer(gl.clone(), self.program.clone(), "a_uv", 2, &arr).unwrap();

        // Uniform data
        // (right now it's just player displacement)
        let u_displacement = gl.get_uniform_location(self.program.as_ref(), "u_displacement").unwrap();
        gl.uniform2f(Some(&u_displacement), (self.time * 0.006).cos() * 0.2, (self.time * 0.01).sin() * 0.2); // todo things like this should go to `update()`

        // Drawing
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT);

        gl.draw_arrays(GL::TRIANGLES, 0, VERTEX_COUNT as i32);
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

// a helper for uploading buffers and setting up attributes
fn setup_buffer(
    gl: Rc<GL>,
    program: Rc<WebGlProgram>,
    attrib: &str,
    size: i32,
    data: &[f32],
) -> Result<(), JsValue> {
    // Create, bind, and upload buffer.
    let buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
    unsafe {
        let array = js_sys::Float32Array::view(data); // memory danger
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &array, GL::STATIC_DRAW);
    }

    // Set up the attribute thingies
    let attrib = gl.get_attrib_location(&program, attrib) as u32;
    gl.vertex_attrib_pointer_with_i32(attrib, size, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(attrib);

    Ok(())
}

use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{ WebGl2RenderingContext as GL, WebGlProgram, WebGlShader, WebGlTexture,
    WebGlUniformLocation, WebGlVertexArrayObject, HtmlCanvasElement, HtmlImageElement };

pub struct Renderer {
    pub gl: Rc<GL>,

    pub texture: WebGlTexture,
    pub vao: WebGlVertexArrayObject,
    pub vertex_count: i32,

    pub program: Rc<WebGlProgram>, // keeping it here for now, later it will be attached to a draw list
    pub u_displacement: WebGlUniformLocation,
    pub u_canvas_size: WebGlUniformLocation,
}

impl Renderer {
    pub fn new(canvas: &HtmlCanvasElement) -> Self {
        // WebGL context
        let gl = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<GL>()
            .unwrap();
        let gl = Rc::new(gl);

        // Shader program
        let vert_shader = Self::compile_shader(&gl, GL::VERTEX_SHADER, include_str!("./shader.vert")).unwrap();
        let frag_shader = Self::compile_shader(&gl, GL::FRAGMENT_SHADER, include_str!("./shader.frag")).unwrap();
        let program = Self::link_program(&gl, &vert_shader, &frag_shader).unwrap();
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
        ).expect("Not a valid texture");

        // VAO and buffer objects
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));
        let vertex_count = 6;

        Self::setup_buffer(gl.clone(), program.clone(), "a_position", 2, &[
            // (hypothetically we could use indexing to cut this down to just /four/ vertices
            // worth of data... but right now i just dont feel like it)
            -60.0, 60.0, // top left
            -60.0, -60.0, // bottom left
            60.0, -60.0, // bottom right
            -60.0, 60.0, // top left
            60.0, -60.0, // bottom right
            60.0, 60.0, // top right
        ]).unwrap();

        Self::setup_buffer(gl.clone(), program.clone(), "a_uv", 2, &[
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
            gl,

            texture,
            vao,
            vertex_count,

            program,
            u_displacement,
            u_canvas_size,
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
}
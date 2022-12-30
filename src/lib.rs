use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as GL;
use web_sys::*; // todo selective imports later

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    init_panic_hook();

    // DOM
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let gl = canvas.get_context("webgl2")?.unwrap().dyn_into::<GL>()?;
    let gl = Rc::new(gl);

    // texture loading
    // todo texture ids
    texture_from_image(gl.clone(), "./test.webp");

    // Shader Program
    let vert_shader = compile_shader(gl.clone(), GL::VERTEX_SHADER, include_str!("./shader.vert"))?;
    let frag_shader = compile_shader(
        gl.clone(),
        GL::FRAGMENT_SHADER,
        include_str!("./shader.frag"),
    )?;
    let program = link_program(gl.clone(), &vert_shader, &frag_shader)?;
    gl.use_program(Some(&program));

    // GL VAOs and VBOs
    const VERTEX_COUNT: usize = 3;
    {
        // Position data
        let positions: [f32; VERTEX_COUNT * 3] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];
        let texture_uv: [f32; VERTEX_COUNT * 2] = [0., 0., 1., 0., 1., 1.];

        let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
        // spooky memory danger (see Float32Array::view documentation)
        unsafe {
            let array = js_sys::Float32Array::view(&positions);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &array, GL::STATIC_DRAW);
        }
            gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);
            gl.enable_vertex_attrib_array(0);
    }

    // Drawing
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(GL::COLOR_BUFFER_BIT);

    gl.draw_arrays(GL::TRIANGLES, 0, VERTEX_COUNT as i32);
    Ok(())
}

pub fn compile_shader(gl: Rc<GL>, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    gl: Rc<GL>,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

pub fn texture_from_image(gl: Rc<GL>, url: &str) {
    let image = Rc::new(RefCell::new(HtmlImageElement::new().unwrap()));
    let image_for_move = Rc::clone(&image);

    let onload = Closure::wrap(Box::new(move || {
        let texture = gl.create_texture();

        gl.active_texture(GL::TEXTURE0); // fixme when we get to actually making stuff, we need to be sure we are using GL::TEXTUREx thing (coz compatibility)
        gl.bind_texture(GL::TEXTURE_2D, texture.as_ref());
        gl.pixel_storei(GL::UNPACK_FLIP_Y_WEBGL, 1);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
        // gl.pixel_storei(GL::UNPACK_FLIP_Y_WEBGL, 1);

        gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
            GL::TEXTURE_2D,
            0,
            GL::RGBA.try_into().unwrap(), // or `as i21` idk
            GL::RGBA,
            GL::UNSIGNED_BYTE,
            &image_for_move.borrow(), // spooky business
        )
        .expect("Not a valid texture");
    }) as Box<dyn Fn()>);

    let image = image.borrow_mut();
    image.set_onload(Some(onload.as_ref().unchecked_ref()));
    image.set_src(url);

    onload.forget();
}

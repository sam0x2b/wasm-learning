#version 300 es
precision mediump float;

uniform vec2 u_displacement;
uniform vec2 u_canvas_size;

in vec2 a_position;
in vec2 a_uv;

out vec2 v_uv;

void main() {
    // We multiply by 2.0 here because clip space has a range of -1.0 to 1.0 (...not 0 to 1.0).
    gl_Position = vec4((a_position + u_displacement) * 2.0 / u_canvas_size, 0.0, 1.0);
    v_uv = a_uv;
}

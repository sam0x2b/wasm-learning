#version 300 es
precision mediump float;

uniform vec2 u_displacement;

in vec3 a_position;
in vec2 a_uv;

out vec3 v_position;
out vec2 v_uv;

void main() {
    gl_Position = vec4(a_position + vec3(u_displacement, 0.0), 1.0);
    v_position = a_position;
    v_uv = a_uv;
}

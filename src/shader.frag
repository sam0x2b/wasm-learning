#version 300 es
precision mediump float;

uniform sampler2D u_texture;

in vec3 v_position;
in vec2 v_uv;

out vec4 f_color;

void main() {
    // f_color = vec4(v_position * 0.5 + 0.5, 1.0);
    f_color = texture(u_texture, v_uv);
}

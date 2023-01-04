#version 300 es
precision mediump float;

uniform sampler2D u_texture;
in vec2 v_uv;
out vec4 f_color;

void main() {
    f_color = texture(u_texture, v_uv);
}

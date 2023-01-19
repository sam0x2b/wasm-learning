#version 300 es
precision mediump float;

in vec2 a_pos;

void main() {
    gl_Position = a_pos;
}
#version 300 es
precision mediump float;

uniform float width;

in vec2 t;
in vec2 u;
in vec2 v;

void main() {
    vec2 d = normalize(length(t)*v + length(v)*t);
    vec2 d_perp = vec2(-d.x, d.y);

    vec2 tu = u - t;
    float r = width / 2.0;

    vec2 shift = r * d * length(tu) * length(d_perp) / dot(tu, d_perp);
    gl_Position = u + shift * (gl_VertexID * 2 - 1);
}
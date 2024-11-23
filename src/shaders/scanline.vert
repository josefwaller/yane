#version 330 core

in vec2 pos;
uniform int scanline;

void main() {
    gl_Position = vec4(
        1 - 2 * pos.x,
        1.0 - 2.0 * (float(scanline) / 240.0),
        0.9,
        1.0
    );
}
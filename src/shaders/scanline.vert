#version 330 core

in vec2 pos;
uniform int scanline;

void main() {
    gl_Position = vec4(
        1 - 2 * pos.x,// + vec2(0, float(scanline) / 240.0),
        1 - 2 * (scanline / 240.0),
        0.0,
        1.0
    );
}
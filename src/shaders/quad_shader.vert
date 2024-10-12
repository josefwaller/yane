#version 330 core

in vec3 pos;
out vec2 UV;

void main() {
    UV = vec2(pos) / 2 + vec2(0.5, 0.5);
    gl_Position = vec4(pos, 1.0);
}
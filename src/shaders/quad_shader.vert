#version 330 core

in vec3 pos;
out vec2 UV;

void main() {
    // UV should be between 0-1 for both X and Y
    UV = vec2(pos / 2) * mat2(1, 0, 0, -1) + vec2(0.5, 0.5);
    gl_Position = vec4(vec2(pos),  0.0 , 1.0);
}
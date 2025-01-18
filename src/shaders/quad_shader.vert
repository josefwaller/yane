#version 330 core

in vec3 pos;
out vec2 UV;

uniform vec2 screenSize;

void main() {
    // Get amount to shift and multiply the UV coords by in order to set the screen resolution to screenSize
    vec2 screenShift = (vec2(256, 240) - vec2(screenSize)) / (2.0 * 256.0);
    vec2 screenMulti = vec2(screenSize) / vec2(256, 240);
    // UV should be between 0-1 for both X and Y
    UV = screenShift + screenMulti * (vec2(pos / 2) * mat2(1, 0, 0, -1) + vec2(0.5, 0.5));
    gl_Position = vec4(vec2(pos),  0.0 , 1.0);
}
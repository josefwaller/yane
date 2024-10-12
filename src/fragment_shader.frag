#version 330 core

flat in int pixelIndex;
// out vec3 color;
uniform vec3 colors[4];
uniform int sprite[128];

layout (location = 0) out vec3 color;

void main() {
    color = vec3(colors[2 * sprite[pixelIndex] + sprite[pixelIndex + 64]]);
}
// Simple shader that just outputs a unform color
#version 330 core
uniform vec3 inColor;

out vec3 outColor;

void main() {
    outColor = inColor;
    gl_FragDepth = 0.9;
}
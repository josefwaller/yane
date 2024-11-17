#version 330 core

uniform vec3 inColor;

out vec3 outColor;

void main() {
    outColor = inColor;
    gl_FragDepth = 0.9;
    // outColor = vec3(gl_FragDepth, 0, 0);
}
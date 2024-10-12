#version 330 core

layout (location = 0) in int oamIndex;

uniform mat3 positionMatrices[64];
uniform mat3 colors;

// out int oamIndex;

void main() {
    // oamIndex = oamIndex;
   gl_Position = vec4(vec3(0, 0, 1.0) * positionMatrices[oamIndex], 1.0);
}
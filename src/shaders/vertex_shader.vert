#version 330 core

layout (location = 0) in int oamIndex;

uniform uint oamData[4 * 64];
uniform mat3 colors;

out int vertOamIndex;

void main() {
    vertOamIndex = oamIndex;
    gl_Position = vec4(
        vec3(
            (float(oamData[4 * oamIndex + 3]) - 128.0) / 128.0,
            (120 - float(oamData[4 * oamIndex])) / 120.0,
            1.0
        ),
    1.0);
}
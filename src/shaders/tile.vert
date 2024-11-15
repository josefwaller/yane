#version 330 core

in vec2 vertexPos;

uniform vec2 position;
uniform bool flipVertical;
uniform bool flipHorizontal;

const vec2 SCREEN_SIZE = vec2(256, 240);
out vec2 UV;

void main() {
    // Invert Y in order match GL window orientation
    mat2 invertY = mat2(
        1, 0,
        0, -1
    );

    mat3 flipX = mat3(
        -1, 0, 1,
        0, 1, 0,
        0, 0, 1
    );
    mat3 flipY = mat3(
        1, 0, 0,
        0, -1, 1,
        0, 0, 1
    );
    gl_Position = vec4(
        2.0 * (position + 8.0 * vertexPos) * invertY / SCREEN_SIZE - vec2(1.0, -1.0),
        0.0,
        1.0
    );
    UV = vec2(vec3(vertexPos, 1) * (flipHorizontal ? flipX : mat3(1.0)) * (flipVertical ? flipY : mat3(1.0)));
}
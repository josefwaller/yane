#version 330 core

in vec2 vertexPos;

uniform vec2 position;
uniform bool flipVertical;
uniform bool flipHorizontal;
uniform int tileIndex;
uniform int oamPaletteIndex;
uniform vec2 sizes;

const vec2 SCREEN_SIZE = vec2(256, 240);
out vec2 UV;
out float tileAddr;
out float paletteIndex;
out float depth;

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
    mat2 matrix = mat2(
        sizes.x, 0,
        0, sizes.y
        // 1, 0,
        // 0, 2
    );
    vec2 finalPos = (position + 8.0 * vertexPos * matrix) * invertY/ SCREEN_SIZE;
    gl_Position = vec4(
        2.0 * finalPos - vec2(1.0, -1.0),
        0.9,
        1.0
    );
    UV = vec2(vec3(vertexPos, 1) * (flipHorizontal ? flipX : mat3(1.0)) * (flipVertical ? flipY : mat3(1.0)));
    tileAddr = float(tileIndex);
    paletteIndex = float(oamPaletteIndex);
    depth = 0.5;
}
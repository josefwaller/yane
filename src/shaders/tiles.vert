#version 330 core

in vec2 vertexPosition;

const int TILE_COUNT = 33 + 64;

uniform ivec2 positions[TILE_COUNT];
uniform int patternIndices[TILE_COUNT];
uniform int paletteIndices[TILE_COUNT];
uniform float depths[TILE_COUNT];
uniform bool flipHorizontal[TILE_COUNT];
uniform bool flipVertical[TILE_COUNT];
uniform int height;

out vec2 UV;
out float tileAddr;
out float paletteIndex;
out float depth;

const mat3 FLIP_X = mat3(
    -1, 0, 1,
    0, 1, 0,
    0, 0, 1
);
const mat3 FLIP_Y = mat3(
    1, 0, 0,
    0, -1, 1,
    0, 0, 1
);
void main() {
    mat2 heightMatrix = mat2(1, 0, 0, height);
    vec2 pos = 8 * heightMatrix * vertexPosition + vec2(positions[gl_InstanceID]);

    gl_Position = vec4(
        -1 + 2 * float(pos.x) / 256.0,
        1 - 2 * float(pos.y) / 240.0,
        depths[gl_InstanceID],
        1);
    UV = vec2(vec3(vertexPosition, 1)
        * (flipHorizontal[gl_InstanceID] ? FLIP_X : mat3(1))
        * (flipVertical[gl_InstanceID] ? FLIP_Y : mat3(1)))
        * heightMatrix;
    tileAddr = float(patternIndices[gl_InstanceID]);
    paletteIndex = float(paletteIndices[gl_InstanceID]);
    depth = depths[gl_InstanceID];
}
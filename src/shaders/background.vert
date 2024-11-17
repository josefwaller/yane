#version 330 core

in vec2 vertexPosition;

uniform ivec2 positions[32];
uniform int patternIndices[32];
uniform int paletteIndices[32];
uniform int scanline;

out vec2 UV;
out float tileAddr;
out float paletteIndex;
out float depth;

void main() {
    vec2 pos = 8 * vertexPosition + vec2(positions[gl_InstanceID]);
    gl_Position = vec4(
        -1 + 2 * float(pos.x) / 256.0,
        1 - 2 * float(pos.y) / 240.0,
        0.8,
        1);
    UV = vec2(vertexPosition);
    tileAddr = float(patternIndices[gl_InstanceID]);
    paletteIndex = float(paletteIndices[gl_InstanceID]);
    depth = 0.8;
}
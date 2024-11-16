#version 330 core

in vec2 position;

uniform int nametableRow[32];
uniform int paletteIndices[32];
uniform int scanline;

out vec2 UV;
out float tileAddr;
out float paletteIndex;

void main() {
    gl_Position = vec4(
        -1 + 2 * (8.0 * (position.x + float(gl_InstanceID)) / 256.0), 
        1 - 2 * ((float(8 * (scanline / 8)) + 8.0 * position.y) / 240.0),
        1,
        1);
    UV = vec2(position);
    tileAddr = float(nametableRow[gl_InstanceID]);
    paletteIndex = float(paletteIndices[gl_InstanceID]);
}
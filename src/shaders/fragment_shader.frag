#version 330 core

flat in int pixelIndex;
flat in int oamIndex;

uniform vec3 palettes[2 * 4 * 4];
uniform int sprite[128];
uniform uint oamData[4 * 64];

layout (location = 0) out vec4 color;

void main() {
    int index = sprite[pixelIndex] + 2 * sprite[pixelIndex + 64];
    if (index == 0) {
        discard;
    }
    int paletteIndex = int(oamData[4 * oamIndex + 2]) % 4;
    color = vec4(palettes[0x10 + 4 * paletteIndex + index], 1.0);
}
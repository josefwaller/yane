#version 330 core

flat in int pixelIndex;
flat in int oamIndex;

uniform vec3 palettes[2 * 4 * 4 * 3];
uniform int sprite[256];
uniform uint oamData[4 * 64];
uniform uint tall_sprites;

layout (location = 0) out vec4 color;

void main() {
    int offset = (tall_sprites == 0u || pixelIndex < 64) ? 0 : 64;
    int index = sprite[offset + pixelIndex] + 2 * sprite[offset + pixelIndex + 64];
    if (index == 0) {
        discard;
    }
    int paletteIndex = int(oamData[4 * oamIndex + 2]) % 4;
    color = vec4(palettes[0x10 + 4 * paletteIndex + index], 1.0);
}
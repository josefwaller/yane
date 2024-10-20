#version 330 core

flat in int pixelIndex;
flat in int oamIndex;
flat in int tileAddr;

layout (std140) uniform ChrRomUBO
{
    int tiles[256];
} ;

uniform vec3 palettes[2 * 4 * 4 * 3];
uniform int sprite[256];
uniform uint oamData[4 * 64];
uniform uint tall_sprites;

layout (location = 0) out vec4 color;

int getBitAt(int i) {
    return int((tiles[0x10 * tileAddr + i / 8] >> (7 - (i % 8))) & 1);
}

void main() {
    int offset = (tall_sprites == 0u || pixelIndex < 64) ? 0 : 64;
    // int index = sprite[offset + pixelIndex] + 2 * sprite[offset + pixelIndex + 64];
    // int index = (int(tiles[1])) + 2 * (int(tiles[0]));
    // int index = int((tiles[pixelIndex / 8] >> (pixelIndex % 8)) & 1);
    int index = getBitAt(offset + pixelIndex) + 2 * getBitAt(offset + pixelIndex + 64);
    // int index = tiles[4] & 0x1;
    // int index = getBitAt(0);
    // int index = (tiles[0] & ~0x0F) == 0 ? 0 : 1;
    // int index = pixelIndex % 2 == 0 ? 0 : 1;
    if (index == 0) {
        discard;
        // index = 1;
    }
    int paletteIndex = int(oamData[4 * oamIndex + 2]) % 4;
    color = vec4(palettes[0x10 + 4 * paletteIndex + index], 1.0);
}

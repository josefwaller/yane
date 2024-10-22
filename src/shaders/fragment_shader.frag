#version 330 core
// The pixel index in the texture (0 is the top left, 7 is the top right, 8 is the top left second row, etc)
flat in int pixelIndex;
// Address of the tile in CHR ROM
flat in int tileAddr;
// Indes of the palette to use
flat in int paletteIndex;
// Texture holding the CHR_ROM values
uniform isampler1D chrRomTex;
// Uniform holding the palettes
uniform vec3 palettes[2 * 4 * 4 * 3];
// 1 if 8x16 is on, 0 otherwise
uniform uint tall_sprites;

layout (location = 0) out vec4 color;

int getBitAt(int i) {
    int texValue = texelFetch(chrRomTex, 0x10 * tileAddr + i / 8, 0).r;
    return (texValue >> (7 - (i % 8))) & 1;
}

void main() {
    int offset = (1 - int(tall_sprites == 0u || pixelIndex < 64)) * 64;
    int index = getBitAt(offset + pixelIndex) + 2 * getBitAt(offset + pixelIndex + 64);
    if (index == 0) {
        discard;
    }
    color = vec4(palettes[0x10 + 4 * paletteIndex + index], 1.0);
}

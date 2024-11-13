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
uniform int palettes[2 * 4 * 4];
// Colors to actually render
uniform vec3 colors[0x40];
// 1 if 8x16 is on, 0 otherwise
uniform int tallSprites;
// Greyscale mode
uniform int greyscaleMode;
// Various tints
uniform int redTint;
uniform int blueTint;
uniform int greenTint;

layout (location = 0) out vec4 color;

int getBitAt(int i) {
    int texValue = texelFetch(chrRomTex, 0x10 * tileAddr + i / 8, 0).r;
    return (texValue >> (7 - (i % 8))) & 1;
}

void main() {
    int offset = (1 - int(tallSprites == 0 || pixelIndex < 64)) * 64;
    int index = getBitAt(offset + pixelIndex) + 2 * getBitAt(offset + pixelIndex + 64);
    if (index == 0) {
        discard;
    }
    int colorIndex = palettes[4 * paletteIndex + index];
    // Turn greyscale if greyscale mode is on
    if (greyscaleMode != 0) {
        colorIndex &= 0x30;
    }
    color = vec4(
        colors[colorIndex].r / (1 + greenTint + blueTint),
        colors[colorIndex].g / (1 + redTint + blueTint),
        colors[colorIndex].b / (1 + redTint + greenTint),
        1.0);
}

#version 330
// Geometry shader that interprets `index` as the index of an object in OAM
// Spawns points accordingly

layout (points) in;
layout (points, max_vertices = 128) out;

in int index[];

uniform uint oamData[4 * 64];
// Whether to hide the pixels in the leftmost 8 pixel columns
uniform int hide_left_sprites;
// Whether to render 8x8 or 8x16 sprites
// True (i.e. not 0) for 8x16
uniform int tallSprites;
// location of sprite pattern table in CHR ROM
uniform int spritePatternLocation;

uniform int scrollX;
uniform int scrollY;

flat out int pixelIndex;
flat out int paletteIndex;
flat out int tileAddr;

bool horizontal_flip(uint attr_byte) {
    return (attr_byte & 0x40u) != 0u;
}
bool vertical_flip(uint attr_byte) {
    return (attr_byte & 0x80u) != 0u;
}

void main() {
    for (int j = 0; j < index.length(); j++) {
        int yMax = tallSprites != 0 ? 16 : 8;
        int i = index[j];
        uint attr_byte = oamData[4 * i + 2];
        for (int y = 0; y < yMax; y++) {
            int yPos = int(oamData[4 * i]) + (vertical_flip(attr_byte) ? 7 - y : y);
            for (int x = 0; x < 8; x++) {
                int xPos = int(oamData[4 * i + 3]) + int(horizontal_flip(attr_byte) ? 7 - x : x);
                if (hide_left_sprites != 0 && xPos < 8) {
                    continue;
                }
                // Screen coords are inbetween [-1.0, 1.0], sprite coords (xPos, yPos) are inbetween [0, 255]
                gl_Position = vec4(
                    float(xPos) / 128.0 - 1.0,
                    1.0 - float(yPos) / 120.0,
                    0,
                    1
                );
                pixelIndex = 8 * y + x;
                tileAddr = spritePatternLocation + int(oamData[4 * i + 1]);
                paletteIndex = 4 + int(oamData[4 * i + 2]) % 4;
                EmitVertex();
            }
        }
        EndPrimitive();
    }
}

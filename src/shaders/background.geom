#version 400
layout (points) in;
layout (points, max_vertices=64) out;

in int index[];

// 0x400 bytes / 4 bytes per i32 = 0x100 i32s
uniform int nametable[0x100];
uniform int backgroundPatternLocation;
uniform int hideLeftmostBackground;
uniform int scrollX;
uniform int scrollY;

flat out int pixelIndex;
flat out int tileAddr;
flat out int paletteIndex;

int getNametableByte(int index) {
    return ((nametable[index / 4] >> (8 * (index % 4)))) & 0xFF;
}

int getPaletteIndex(int i) {
    // int attr_addr = 0x3C2;
    // // int attr_addr = 0x03C0 + 8 * (index / 0x20) + 3;//(index % 0x10);// (index % 0x20) / 2;
    int attr_addr = 0x3C0 + 8 * (i / 0x80) + ((i % 0x20) / 4) ;
    // int attr_byte = getNametableByte(attr_addr);
    int attr_byte = getNametableByte(attr_addr);
    // // return (attr_byte & 0xC) >> 2;
    int x = (i / 2) % 2;
    // // int y = (index % 0x20) / 0x10;
    int y = (i / 0x40) % 2;
    // // return (attr_byte >> (2 * ((index % 32) / 16 + (index % 16)))) & 0x03;
    return (attr_byte >> (2 * (2 * y + x))) & 0x03;
    // return 2;
}

void main() {
    for (int i = 0; i < index.length(); i++) {
        int j = index[0];
        int xPos = j % 32;
        int yPos = j / 32;
        for (int x = 0; x < 8; x++) {
            if (8 * xPos + x < 8 && hideLeftmostBackground != 0) {
                continue;
            }
            for (int y = 0; y < 8; y++) {
                gl_Position = vec4(
                    (8.0 * xPos + x + scrollX) / 128.0 - 1.0,
                    1 - (8.0 * yPos + y + scrollY) / 120.0,
                    -1,
                    1);
                pixelIndex = 8 * y + x;
                // Divide by 0x10 since each tile is 16 bytes long
                tileAddr = backgroundPatternLocation / 0x10 + getNametableByte(j);
                paletteIndex = getPaletteIndex(j);
                EmitVertex();
            }
        }
        EndPrimitive();
    }
}
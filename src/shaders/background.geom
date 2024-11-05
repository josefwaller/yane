#version 400
layout (points) in;
layout (points, max_vertices=64) out;

in int index[];

// 0x400 bytes / 4 bytes per i32 = 0x100 i32s
// 2 nametables so 0x200
uniform int nametable[0x200];
uniform int backgroundPatternLocation;
uniform int hideLeftmostBackground;
uniform int scrollX;
uniform int scrollY;

flat out int pixelIndex;
flat out int tileAddr;
flat out int paletteIndex;

int getNametableByte(int index) {
    return (nametable[index / 4] >> (8 * (index % 4))) & 0xFF;
}

int getPaletteIndex(int i) {
    // Start of the nametable's attribute table
    int attr_start = 0x3C0 + 0x400 * (i / (32 * 30));
    int tile_index = i % (32 * 30);
    // Get byte in the nametable attribute for this tile
    int attr_addr = attr_start + 8 * (tile_index / 0x80) + ((tile_index % 0x20) / 4) ;
    int attr_byte = getNametableByte(attr_addr);
    // Parse X/Y coord in group of 4 tiles
    int x = (tile_index / 2) % 2;
    int y = (tile_index / 0x40) % 2;
    // Return value for this tile specifically
    return (attr_byte >> (2 * (2 * y + x))) & 0x03;
}

void main() {
    for (int i = 0; i < index.length(); i++) {
        int j = index[0];
        bool isSecondNametable = j >= 32 * 30;
        // TODO: Mirroring
        int xPos = isSecondNametable ? 32 + (j % 32) : j % 32 ;
        // TODO: Find out why 2 here
        int yPos = isSecondNametable ? (j - (32 * 30)) / 32: j / 32;
        for (int x = 0; x < 8; x++) {
            if (8 * xPos + x < 8 && hideLeftmostBackground != 0) {
                continue;
            }
            for (int y = 0; y < 8; y++) {
                gl_Position = vec4(
                    (8.0 * xPos + x - scrollX) / 128.0 - 1.0,
                    1 - (8.0 * yPos + y - scrollY) / 120.0,
                    -1,
                    1);
                pixelIndex = 8 * y + x;
                // If we're in the second table, account for the attribute table
                int tileIndex = isSecondNametable ? j + 64 : j;
                // Divide by 0x10 since each tile is 16 bytes long
                tileAddr = backgroundPatternLocation / 0x10 + getNametableByte(tileIndex);
                paletteIndex = getPaletteIndex(j);
                EmitVertex();
            }
        }
        EndPrimitive();
    }
}
#version 400
// Shader that simple draws the tile at `index` in CHR ROM
// Used in the debug window
layout(points) in;
layout(points, max_vertices = 64) out;

in int[] index;

flat out int pixelIndex;
flat out int tileAddr;
flat out int paletteIndex;

void main() {
    for (int i = 0; i < index.length(); i++) {
        for (int y = 0; y < 8; y++) {
            for (int x = 0; x < 8; x++) {
                int xPos = 8 * (index[i] % 32) + x;
                int yPos = 8 * (index[i] / 32) + y;
                // Todo, maybe set this in uniform
                gl_Position = vec4(
                    float(xPos) / 128.0 - 1.0,
                    1 - float(yPos) / 120.0,
                    0,
                    1
                );
                pixelIndex = 8 * y + x;
                tileAddr = index[i];
                paletteIndex = 0;
                EmitVertex();
            }
        }
    }
    EndPrimitive();
}
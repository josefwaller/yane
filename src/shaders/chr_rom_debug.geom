#version 400
// Shader that simple draws the tile at `index` in CHR ROM
// Used in the debug window
layout(points) in;
layout(points, max_vertices = 64) out;

in int[] index;

uniform int numRows;
uniform int numColumns;

flat out int pixelIndex;
flat out int tileAddr;
flat out int paletteIndex;

void main() {
    for (int i = 0; i < index.length(); i++) {
        for (int y = 0; y < 8; y++) {
            for (int x = 0; x < 8; x++) {
                int xPos = 8 * (index[i] % numColumns) + x;
                int yPos = 8 * (index[i] / numColumns) + y;
                gl_Position = vec4(
                    2 * (float(xPos) / (8.0 * float(numColumns)))- 1.0,
                    1 - 2 * (float(yPos) / (8 * float(numRows))),
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
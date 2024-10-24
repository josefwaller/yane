#version 400
layout (points) in;
layout (points, max_vertices=150) out;

in int index[];

uniform int nametable[0x400];

flat out int pixelIndex;
flat out int tileAddr;
flat out int paletteIndex;

void main() {
    for (int i = 0; i < index.length(); i++) {
        int j = index[0];
        int xPos = j % 32;
        int yPos = j / 32;
        for (int x = 0; x < 8; x++) {
            for (int y = 0; y < 8; y++) {
                gl_Position = vec4(
                    (8.0 * xPos + x) / 128.0 - 1.0,
                    1 - (8.0 * yPos + y) / 120.0,
                    -1,
                    1);
                pixelIndex = 8 * y + x;
                tileAddr = nametable[j];
                paletteIndex = 0;
                EmitVertex();
            }
        }
        EndPrimitive();
    }
}
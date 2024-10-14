#version 330
layout (points) in;
layout (points, max_vertices = 64) out;

in int vertOamIndex[];

uniform mat3 colors;
uniform uint oamData[4 * 64];
// Whether to hide the pixels in the leftmost 8 pixel columns
uniform uint hide_left_sprites;

flat out int pixelIndex;
flat out int oamIndex;

void main() {
    for (int y = 0; y < 8; y++) {
        for (int x = 0; x < 8; x++) {
            int yPos = int(oamData[4 * oamIndex]) + ((oamData[4 * oamIndex + 2] & 0x80u) == 0u ? y : 7 - y);
            int xPos = int(oamData[4 * oamIndex + 3]) + ((oamData[4 * oamIndex + 2] & 0x40u) == 0u ? x : 7 - x);
            gl_Position = vec4(
                float(xPos) / 128.0 - 1.0,
                1.0 - float(yPos) / 120.0,
                0,
                1
            );
            pixelIndex = 8 * y + x;
            oamIndex = vertOamIndex[0];
            EmitVertex();
        }
    }
    EndPrimitive();
}
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

bool horizontal_flip(uint attr_byte) {
    return (attr_byte & 0x40u) != 0u;
}
bool vertical_flip(uint attr_byte) {
    return (attr_byte & 0x80u) != 0u;
}

void main() {
    for (int y = 0; y < 8; y++) {
        for (int x = 0; x < 8; x++) {
            int i = vertOamIndex[0];
            uint attr_byte = oamData[4 * i + 2];
            int yPos = int(oamData[4 * i]) + (vertical_flip(attr_byte) ? 7 - y : y);
            int xPos = int(oamData[4 * i + 3]) + int(horizontal_flip(attr_byte) ? 7 - x : x);
            if (hide_left_sprites != 0u && xPos < 8) {
                continue;
            }
            // Screen coords are inbetween [-1.0, 1.0], sprite coords (xPos, yPos) are inbetween [0, 255]
            gl_Position = vec4(
                float(xPos) / 128.0 - 1.0,
                1.0 - float(yPos) / 120.0,
                1,
                1
            );
            pixelIndex = 8 * y + x;
            oamIndex = vertOamIndex[0];
            EmitVertex();
        }
    }
    EndPrimitive();
}

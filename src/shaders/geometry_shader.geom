#version 330
layout (points) in;
layout (points, max_vertices = 64) out;

in int vertOamIndex[];

uniform mat3 colors;
uniform uint oamData[4 * 64];

flat out int pixelIndex;
flat out int oamIndex;

void main() {
    for (int y = 0; y < 8; y++) {
        for (int x = 0; x < 8; x++) {
            int yPos = (int(oamData[4 * oamIndex + 2]) & 0x80) == 0 ? y : 7 - y;
            int xPos = (int(oamData[4 * oamIndex + 2]) & 0x40) == 0 ? x : 7 - x;
            gl_Position = gl_in[0].gl_Position + vec4(
                float(xPos) / 128.0,
                -float(yPos) / 120.0,
                0,
                0
            );
            pixelIndex = 8 * y + x;
            oamIndex = vertOamIndex[0];
            EmitVertex();
        }
    }
    EndPrimitive();
}
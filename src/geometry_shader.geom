#version 330
//in vec3 pos;
// out vec3 aPos;
// in vec3 outColor;
// out vec3 outColor;
layout (points) in;
layout (points, max_vertices = 64) out;

uniform mat3 colors;

flat out int pixelIndex;

void main() {
    // outColor = outColor;
    for (int y = 0; y < 8; y++) {
        for (int x = 0; x < 8; x++) {
            gl_Position = gl_in[0].gl_Position + vec4(float(x) / 128.0, float(y) / 120.0, 0, 0);
            pixelIndex = 8 * y + x;
            EmitVertex();
        }
    }
    EndPrimitive();
}
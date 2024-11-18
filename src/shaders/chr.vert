// Just draws a section of CHR ROM
#version 330 core
in vec2 vertexPosition;

uniform int numColumns;
uniform int numRows;
uniform int tileOffset;

out float tileAddr;
out float paletteIndex;
out float depth;
out vec2 UV;

void main() {
    // These are not used by the debug window
    paletteIndex = 0.0;
    depth = 0.5;
    // This is
    tileAddr = float(tileOffset + gl_InstanceID);

    vec2 position = vec2(
        8 * (gl_InstanceID % numColumns + vertexPosition.x),
        8 * (gl_InstanceID / numRows + vertexPosition.y)
    );
    gl_Position = vec4(
        -1 + 2 * position.x / (8 * numColumns),
        1 - 2 * position.y / (8 * numRows),
        0.8,
        1.0
    );
    UV = vertexPosition;
}
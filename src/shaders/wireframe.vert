#version 330 core

in vec2 vertexPos;

uniform vec2 position;
uniform vec2 sizes;
uniform vec2 screenSize;

const vec2 OUTPUT_SIZE = vec2(256, 240);

void main() {
    // Invert Y in order match GL window orientation
    mat2 invertY = mat2(
        1, 0,
        0, -1
    );

    mat2 matrix = mat2(
        sizes.x, 0,
        0, sizes.y
    );
    vec2 gamePos = (position + 8.0 * vertexPos * matrix) * invertY / OUTPUT_SIZE;
    vec2 screenScale = OUTPUT_SIZE / screenSize;
    // Zoom in to fit screen size
    gl_Position = vec4(
        screenScale * (2.0 * gamePos - vec2(1.0, -1.0)),
        0.9,
        1.0
    );
}
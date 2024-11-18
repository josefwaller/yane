#version 330 core

in vec2 vertexPos;

uniform vec2 position;
uniform vec2 sizes;

const vec2 SCREEN_SIZE = vec2(256, 240);

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
    vec2 gamePos = (position + 8.0 * vertexPos * matrix) * invertY / SCREEN_SIZE;
    gl_Position = vec4(
        2.0 * gamePos - vec2(1.0, -1.0),
        0.9,
        1.0
    );
}
#version 330 core

layout (location = 0) in int oamIndex;

out int index;

void main() {
    index = oamIndex;
    gl_Position = vec4(
        vec3(
            0,
            0,
            1.0
        ),
    1.0);
}
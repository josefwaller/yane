#version 330

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 normal;

uniform mat3 rotation;
uniform mat3 scale;
uniform mat4 translation;

out vec2 texCoords;
out vec3 vertexPosition;
out vec3 vertexNormal;

void main() {
    vec4 pos = translation * vec4(scale * pos * rotation, 1.0);
    gl_Position = pos;
    vertexPosition = vec3(pos);
    texCoords = vec2(1.0, -1.0) * uv;
    vertexNormal = rotation * normal;
}
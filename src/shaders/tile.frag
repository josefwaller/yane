#version 330 core

in vec2 UV;

out vec4 color;

uniform sampler2D chrTex;
uniform int tileIndex;
uniform vec3 palette[4];

void main() {
    int index = int(texelFetch(chrTex, ivec2(0, 8 * tileIndex) + ivec2(8.0 * UV), 0).r * 256.0);
    if (index == 0) {
        discard;
    }
    color = vec4(palette[index], 1.0);
}
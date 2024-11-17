#version 330 core

in vec2 UV;

uniform vec3 palette[32];
uniform sampler2D chrTex;

// this should actually be an int
in float tileAddr;
in float paletteIndex;
in float depth;

out vec4 color;


void main() {
    int index = int(texelFetch(chrTex, ivec2(0, 8 * int(tileAddr)) + ivec2(floor(8.0 * UV)), 0).r * 256.0);
    // int index = int(texelFetch(chrTex, ivec2(0, 8 * int(tileAddr)) + ivec2(floor(vec2(5, 4))), 0).r * 256.0);
    // int index = 1;
    if (index == 0) {
        discard;
    }
    color = vec4(palette[4 * int(paletteIndex) + int(index)], 1.0);
    // color = vec4(UV, 0, 1);
    gl_FragDepth = depth;
    // color = vec4(gl_FragDepth, 0, 0, 1);
}
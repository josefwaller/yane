#version 330

in vec2 texCoords;
in vec3 vertexPosition;
in vec3 vertexNormal;

uniform sampler2D tex;
uniform sampler2D bumpmap;
uniform vec3 lightPosition;
uniform float ambient;
uniform float lightStrength;

out vec4 outColor;

void main() {
    vec3 lightVec = normalize(lightPosition - vertexPosition);
    // vec3 normal = normalize(vec3(texture(bumpmap, texCoords)));
    vec3 vNormal = normalize(vertexNormal);
    // float diffuse = max(dot(normalize(lightPosition - vec3(gl_FragCoord)), normalize(vec3(texture(bumpmap, texCoords)))), 0.0);
    float diffuse = ambient + max(lightStrength * dot(vNormal, lightVec), 0.0);
    outColor = vec4(diffuse * vec3(texture(tex, texCoords)), 1.0);
    // outColor = vec4(vec3(0.5) + vNormal / 2.0, 1.0);
}
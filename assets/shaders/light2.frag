precision mediump float;

const float TILE_SIZE = 32.0;
const float CHUNK_SIZE = 16.0;
const float LOCAL_SIZE = 8.0;

uniform sampler2D depthTex;
uniform sampler2D entityDepthTex;
uniform vec2 cameraPos;
uniform vec2 cameraSize;

varying float radius;
varying vec3 color;
varying vec2 localPos;
varying vec3 localCenter;

void main(void) {
    vec2 texCoord = localPos / cameraSize;
    texCoord.y = 1.0 - texCoord.y;


    vec3 localPos3;

    float entityDepth = texture2D(entityDepthTex, texCoord).r;

    vec2 worldPos = localPos + cameraPos;

    if (entityDepth > 0.0) {
        float z = entityDepth * (CHUNK_SIZE * TILE_SIZE + 2.0 * 64.0) - 64.0;
        localPos3 = vec3(localPos.x, localPos.y + z, z);
    } else {
        float depth = texture2D(depthTex, texCoord).r;
        float z = depth * CHUNK_SIZE * TILE_SIZE;
        localPos3 = vec3(localPos.x, localPos.y + z, z);
    }

    vec3 off = localPos3 - localCenter;
    float dist = length(off);

    float ratio = 1.0 - (dist * dist) / (radius * radius);
    gl_FragColor = vec4(color * ratio, ratio);
}

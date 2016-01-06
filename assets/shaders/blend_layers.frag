precision mediump float;

uniform vec2 cameraPos;
uniform vec2 cameraSize;
uniform vec2 sliceCenter;
uniform float sliceZ;

uniform sampler2D baseTex;
uniform sampler2D slicedTex;
uniform sampler2D cavernTex;

varying vec2 texCoord;

#include "slicing.inc"

float rescale(float x, float min_, float max_) {
    float y = (clamp(x, -1.0, 1.0) + 1.0) / 2.0;
    return y * (max_ - min_) + min_;
}

void main(void) {
    vec2 pixelPos = sliceGetPos();
    float inside = sliceCalcInside(pixelPos);

    vec4 baseColor = texture2D(baseTex, texCoord);
    vec4 slicedColor = texture2D(slicedTex, texCoord);
    gl_FragColor = mix(baseColor, slicedColor, rescale(inside, 0.0, 0.8));
}

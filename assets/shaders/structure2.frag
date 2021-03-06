precision highp float;

#extension GL_EXT_draw_buffers : enable

#ifdef GL_EXT_draw_buffers
# define emit(idx, val)   gl_FragData[(idx)] = (val)
#else
# define emit(idx, val)   if (idx == OUTPUT_IDX) gl_FragData[0] = (val)
#endif

uniform sampler2D sheetTex;
uniform sampler2D cavernTex;
uniform vec2 cameraPos;
uniform vec2 cameraSize;
uniform vec2 sliceCenter;
uniform float sliceZ;

varying vec2 texCoord;
varying float baseZ;

#include "slicing.inc"

void main(void) {
    if (sliceCheck()) {
        discard;
    }

    vec4 color = texture2D(sheetTex, texCoord);
#ifndef OUTPOST_SHADOW
    if (color.a < 1.0) {
        discard;
    } else {
        emit(0, color);
        float tileZ = baseZ;
        emit(1, vec4(tileZ * 8.0 / 255.0, 0.0, 1.0, 1.0));
    }
#else
    if (color.a == 0.0 || color.a == 1.0) {
        discard;
    } else {
        emit(0, color);
    }
#endif
}

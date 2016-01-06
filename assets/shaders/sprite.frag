precision mediump float;

#extension GL_EXT_draw_buffers : enable

#ifdef GL_EXT_draw_buffers
# define emit(idx, val)   gl_FragData[(idx)] = (val)
#else
# define emit(idx, val)   if (idx == OUTPUT_IDX) gl_FragData[0] = (val)
#endif

uniform vec2 cameraPos;
uniform vec2 cameraSize;
uniform vec2 sliceCenter;
uniform float sliceZ;
uniform sampler2D cavernTex;

uniform sampler2D imageTex;

varying highp vec2 normalizedTexCoord;
varying float baseZ;

#include "slicing.inc"

void main(void) {
    if (sliceCheck()) {
        discard;
    }

    vec4 color = texture2D(imageTex, normalizedTexCoord);
    if (color.a == 0.0) {
        discard;
    }
    emit(0, color);
    emit(1, vec4(0.0, 0.0, 0.0, 1.0));
}

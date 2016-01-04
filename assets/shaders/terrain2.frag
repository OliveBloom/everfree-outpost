precision mediump float;

#extension GL_EXT_draw_buffers : enable

#ifdef GL_EXT_draw_buffers
# define emit(idx, val)   gl_FragData[(idx)] = (val)
#else
# define emit(idx, val)   if (idx == OUTPUT_IDX) gl_FragData[0] = (val)
#endif

uniform sampler2D atlasTex;
uniform sampler2D cavernTex;
uniform vec2 cameraPos;
uniform vec2 cameraSize;
uniform vec2 sliceCenter;
uniform float sliceZ;

varying vec2 texCoord;
varying float baseZ;

#include "slicing.inc"

void main(void) {
    if (check_slice()) {
        discard;
    }

    vec4 color = texture2D(atlasTex, texCoord);
    if (color.a == 0.0) {
        discard;
    } else {
        emit(0, color);
        emit(1, vec4(0.0, 0.0, 0.0, 1.0));
    }
}

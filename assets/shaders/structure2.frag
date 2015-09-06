precision mediump float;

const float TILE_SIZE = 32.0;
const float CHUNK_SIZE = 16.0;

#extension GL_EXT_frag_depth : enable
#extension GL_EXT_draw_buffers : enable

#ifdef GL_EXT_draw_buffers
# define emit(idx, val)   gl_FragData[(idx)] = (val)
#else
# define emit(idx, val)   if (idx == OUTPUT_IDX) gl_FragData[0] = (val)
#endif

uniform sampler2D sheetTex;
uniform sampler2D depthTex;

varying vec2 texCoord;
varying float baseZ;

void main(void) {
    vec4 color = texture2D(sheetTex, texCoord);
    if (color.a < 1.0) {
        discard;
    } else {
        emit(0, color);
        float tileZ = baseZ / 32.0;
        emit(1, vec4(tileZ * 8.0 / 255.0, 0.0, 1.0, 1.0));
    }
    // gl_FragCoord.z steps by 1/512, while color values step by 1/255.  Note
    // that gl_FragCoord varies in the range 0..1, not -1..+1
    gl_FragDepthEXT = gl_FragCoord.z -
        (255.0 / (CHUNK_SIZE * TILE_SIZE)) * texture2D(depthTex, texCoord).r;
}

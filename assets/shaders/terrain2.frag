precision mediump float;

#extension GL_EXT_draw_buffers : enable

#ifdef GL_EXT_draw_buffers
# define emit(idx, val)   gl_FragData[(idx)] = (val)
#else
# define emit(idx, val)   if (idx == OUTPUT_IDX) gl_FragData[0] = (val)
#endif

const float TILE_SIZE = 32.0;
const float CAVERN_MAP_RADIUS = 32.0;
const float CAVERN_MAP_SIZE = CAVERN_MAP_RADIUS * 2.0 + 4.0;
const float CAVERN_STEP = 1.0 / CAVERN_MAP_SIZE;
const float CAVERN_SLICED = 3.0 / 255.0;

uniform sampler2D atlasTex;
uniform sampler2D cavernTex;
uniform vec2 cameraPos;
uniform vec2 cameraSize;
uniform vec2 sliceCenter;
uniform float sliceZ;

varying vec2 texCoord;
varying float baseZ;

void main(void) {
    if (baseZ >= sliceZ) {
        float pixelX = cameraPos.x + gl_FragCoord.x;
        float pixelY = cameraPos.y + cameraSize.y - gl_FragCoord.y + baseZ * TILE_SIZE;
        vec2 pixelPos = vec2(pixelX, pixelY);
        //vec2 tilePos = floor(pixelPos / TILE_SIZE);
        vec2 tilePos = pixelPos / TILE_SIZE;
        vec2 pixelOffset = pixelPos - tilePos * TILE_SIZE;
        vec2 cavernPos = tilePos - sliceCenter;

        vec2 cavernTexCoord = cavernPos / CAVERN_MAP_SIZE + 0.5;
        float centerVal = texture2D(cavernTex, cavernTexCoord).r;
        if (centerVal >= 1.5 / 255.0) {
            discard;
        }
    }

    vec4 color = texture2D(atlasTex, texCoord);
    if (color.a == 0.0) {
        discard;
    } else {
        emit(0, color);
        emit(1, vec4(0.0, 0.0, 0.0, 1.0));
    }
    //gl_FragData[0] = vec4(1.0, 0.0, 0.0, 1.0);
}

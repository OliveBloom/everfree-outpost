precision mediump float;

const float SHEET_SIZE = 2048.0;
const float ANIM_MODULUS_MS = 55440.0;

uniform vec2 camera_pos;
uniform vec2 camera_size;
uniform vec2 sliceCenter;
uniform float sliceZ;
uniform float now;  // Seconds
uniform sampler2D cavernTex;

attribute vec2 dest_pos;
attribute vec2 src_pos;
attribute float sheet;
attribute vec3 color;
attribute vec4 ref_pos_size;
attribute vec4 anim_info;

varying vec2 tex_coord;
varying vec3 ref_pos;
varying float ref_size_z;
varying vec3 color_;


//const float TILE_SIZE = 32.0; // redefined in slicing.inc
const float CHUNK_SIZE = 16.0;
const float LOCAL_SIZE = 8.0;

// TODO: gross hacks involving interaction between slicing.inc and this shader
// (need to implement proper support for vertex-shader slicing)
#define cameraPos camera_pos
#define cameraSize camera_size
#define gl_FragCoord vec4(0.0)
#define baseZ 0.0
#include "slicing.inc"

const float WRAP_MARGIN = TILE_SIZE * CHUNK_SIZE;
const float WRAP_STEP = TILE_SIZE * CHUNK_SIZE * LOCAL_SIZE;




void main(void) {
    ref_pos = ref_pos_size.xyz;
    ref_size_z = ref_pos_size.w;

    vec2 pos = dest_pos;

    // If it's too far left/up from the camera, wrap around.
    if (ref_pos.x < camera_pos.x - WRAP_MARGIN) {
        ref_pos.x += WRAP_STEP;
        pos.x += WRAP_STEP;
    } else if (ref_pos.x > camera_pos.x + camera_size.x + WRAP_MARGIN) {
        ref_pos.x -= WRAP_STEP;
        pos.x -= WRAP_STEP;
    }
    if (ref_pos.y < camera_pos.y - WRAP_MARGIN) {
        ref_pos.y += WRAP_STEP;
        pos.y += WRAP_STEP;
    } else if (ref_pos.y > camera_pos.y + camera_size.y + WRAP_MARGIN) {
        ref_pos.y -= WRAP_STEP;
        pos.y -= WRAP_STEP;
    }

    vec2 norm_pos = (pos - camera_pos) / camera_size;
    float norm_depth = (ref_pos.y - ref_pos.z) / (TILE_SIZE * CHUNK_SIZE * LOCAL_SIZE * 2.0);

    if (ref_pos.z >= (sliceZ + 1.5) * TILE_SIZE &&
            sliceCalcInside(ref_pos.xy) >= 0.0) {
        norm_depth = -100.0;
    }

    vec3 adj_pos = vec3(norm_pos, norm_depth) * 2.0 - 1.0;
    adj_pos.y = -adj_pos.y;
    gl_Position = vec4(adj_pos, 1.0);


    vec2 tex_pos = src_pos;

    float anim_length = anim_info.x;
    float anim_rate = anim_info.y;
    float anim_step = anim_info.w;

    float frame = mod(floor(now * anim_rate), anim_length);
    // Weird, but it seems like frame == anim_length when it ought to be zero.
    if (frame < anim_length) {
        tex_pos.x += frame * anim_step;
    }

    tex_coord = tex_pos / SHEET_SIZE;


    color_ = color;
}

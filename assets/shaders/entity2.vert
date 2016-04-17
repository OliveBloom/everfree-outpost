precision mediump float;

const float TILE_SIZE = 32.0;
const float CHUNK_SIZE = 16.0;
const float LOCAL_SIZE = 8.0;

const float WRAP_MARGIN = TILE_SIZE * CHUNK_SIZE;
const float WRAP_STEP = TILE_SIZE * CHUNK_SIZE * LOCAL_SIZE;

const float SHEET_SIZE = 2048.0;
const float ANIM_MODULUS_MS = 55440.0;

uniform vec2 camera_pos;
uniform vec2 camera_size;
uniform float now;  // Seconds

attribute vec2 dest_pos;
attribute vec2 src_pos;
attribute float sheet;
attribute vec4 ref_pos_size;
attribute float anim_length;
attribute float anim_rate;
attribute float anim_start;
attribute float anim_step;

varying vec2 tex_coord;
varying vec3 ref_pos;
varying float ref_size_z;


void main(void) {
    ref_pos = ref_pos_size.xyz;
    ref_size_z = ref_pos_size.w;

    vec2 pos = dest_pos;

    // If it's too far left/up from the camera, wrap around.
    if (ref_pos.x < camera_pos.x - WRAP_MARGIN) {
        ref_pos.x += WRAP_STEP;
        pos.x += WRAP_STEP;
    }
    if (ref_pos.y < camera_pos.y - WRAP_MARGIN) {
        ref_pos.y += WRAP_STEP;
        pos.y += WRAP_STEP;
    }

    vec2 norm_pos = (pos - camera_pos) / camera_size;
    float norm_depth = 0.5; //(ref_pos.y - ref_pos.z) / (TILE_SIZE * CHUNK_SIZE * LOCAL_SIZE * 2.0);

    vec3 adj_pos = vec3(norm_pos, norm_depth) * 2.0 - 1.0;
    adj_pos.y = -adj_pos.y;
    gl_Position = vec4(adj_pos, 1.0);


    tex_coord = src_pos / SHEET_SIZE;

    /*
    vec2 texPx = displayOffset + vec2(vertOffset.x, vertOffset.y - vertOffset.z);

    if (animLength != 0.0) {
        float frame;
        if (animLength >= 0.0) {
            frame = mod(floor(now * animRate), animLength);
        } else {
            // Compute the delta in milliseconds between `now` and
            // `animOneshotStart`, in the range -MODULUS/2 .. MODULUS / 2.
            const float HALF_MOD = ANIM_MODULUS_MS / 2.0;
            float now_ms = mod(now * 1000.0, ANIM_MODULUS_MS);
            float delta = mod(now_ms - animOneshotStart + HALF_MOD, ANIM_MODULUS_MS) - HALF_MOD;
            frame = clamp(floor(delta / 1000.0 * animRate), 0.0, -animLength - 1.0);
        }
        texPx.x += frame * animStep;
    }

    texCoord = texPx / (ATLAS_SIZE * TILE_SIZE);
    baseZ = blockPos.z;
    */
}

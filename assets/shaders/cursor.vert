precision mediump float;

const float TILE_SIZE = 32.0;
const float CHUNK_SIZE = 16.0;
const float LOCAL_SIZE = 8.0;

const float WRAP_MARGIN = TILE_SIZE * CHUNK_SIZE;
const float WRAP_STEP = TILE_SIZE * CHUNK_SIZE * LOCAL_SIZE;

attribute vec2 corner;

uniform vec2 camera_pos;
uniform vec2 camera_size;
uniform vec2 cursor_pos;

varying highp vec2 pixel_offset;

void main(void) {
    vec2 signed = corner * 2.0 - 1.0;
    pixel_offset = signed * (TILE_SIZE / 2.0 + 1.0);

    vec2 pos = cursor_pos;
    // If it's too far left/up from the camera, wrap around.
    if (pos.x < camera_pos.x - WRAP_MARGIN) {
        pos.x += WRAP_STEP;
    }
    if (pos.y < camera_pos.y - WRAP_MARGIN) {
        pos.y += WRAP_STEP;
    }

    vec2 norm_pos = (pos + pixel_offset - camera_pos) / camera_size;
    vec2 adj_pos = norm_pos * 2.0 - 1.0;
    adj_pos.y = -adj_pos.y;

    gl_Position = vec4(adj_pos, 0.5, 1.0);
}

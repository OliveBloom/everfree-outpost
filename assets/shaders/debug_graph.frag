precision mediump float;

const float MAX_TIME = 1024.0;
const vec4 BLACK = vec4(20.0, 12.0, 28.0, 255.0) / 255.0;
const vec4 RED = vec4(208.0, 70.0, 72.0, 255.0) / 255.0;
const vec4 YELLOW = vec4(218.0, 212.0, 94.0, 255.0) / 255.0;

uniform vec2 graph_size;
uniform float cur_index;
uniform sampler2D data_tex;

varying vec2 pixel_pos;

void main(void) {
    vec4 data = texture2D(data_tex, vec2(pixel_pos.x / graph_size.x, 0.5));
    float time = (data.x + data.y * 256.0) * 256.0;
    float interval = (data.z + data.w * 256.0) * 256.0;

    float idx = floor(pixel_pos.x);
    if (idx == cur_index) {
        gl_FragColor = BLACK;
        return;
    }

    float frac = 1.0 - pixel_pos.y / graph_size.y;
    float cutoff = frac * log(MAX_TIME);
    if (log(time) > cutoff) {
        gl_FragColor = RED;
    } else if (log(interval) > cutoff) {
        gl_FragColor = YELLOW;
    } else {
        gl_FragColor = BLACK;
    }

    if (log(16.67) > cutoff) {
        gl_FragColor *= vec4(0.6, 0.6, 0.6, 1.0);
    }
}

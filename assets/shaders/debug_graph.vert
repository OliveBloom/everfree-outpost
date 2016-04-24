precision mediump float;

uniform vec2 screen_size;
uniform vec2 graph_pos;
uniform vec2 graph_size;

attribute vec2 corner;

varying vec2 pixel_pos;

void main(void) {
    vec2 pos = (graph_pos + corner * graph_size) / screen_size * 2.0 - 1.0;
    pos.y = -pos.y;
    gl_Position = vec4(pos, 0.0, 1.0);

    pixel_pos = corner * graph_size;
}

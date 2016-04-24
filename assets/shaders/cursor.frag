precision mediump float;

const float TILE_SIZE = 32.0;

varying highp vec2 pixel_offset;

void main(void) {
    float dist = max(abs(pixel_offset.x), abs(pixel_offset.y));
    if (dist >= TILE_SIZE / 2.0 - 1.0) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        discard;
    }
}

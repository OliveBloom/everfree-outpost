precision mediump float;

attribute vec2 corner;

varying vec2 tex_coord;

const mat4 transform = mat4(
        2.0,  0.0,  0.0,  0.0,
        0.0,  2.0,  0.0,  0.0,
        0.0,  0.0,  1.0,  0.0,
       -1.0, -1.0,  0.0,  1.0
       );

void main(void) {
    gl_Position = transform * vec4(corner, 0.0, 1.0);
    tex_coord = corner;
}

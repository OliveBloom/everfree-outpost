precision mediump float;

varying vec2 tex_coord;

uniform sampler2D image_tex;

void main(void) {
    gl_FragColor = texture2D(image_tex, tex_coord);
}

precision mediump float;

uniform sampler2D sheet_tex;
uniform sampler2D depth_tex;
uniform vec2 camera_pos;
uniform vec2 camera_size;
//uniform vec2 sliceCenter;
//uniform float sliceZ;

varying vec2 tex_coord;
varying vec3 ref_pos;
varying float ref_size_z;
varying vec3 color_;

//#include "slicing.inc"

void main(void) {
    //if (sliceCheck()) {
    //    discard;
    //}

    // TODO: depth check

    vec4 base_color = texture2D(sheet_tex, tex_coord);
    if (base_color.a == 0.0) {
        discard;
    } else {
        gl_FragColor = base_color * vec4(color_, 1.0);
    }
}

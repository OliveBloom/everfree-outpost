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

//#include "slicing.inc"

void main(void) {
    //if (sliceCheck()) {
    //    discard;
    //}

    // TODO: depth check

    vec4 color = texture2D(sheet_tex, tex_coord);
    if (color.a == 0.0) {
        //discard;
        gl_FragColor = vec4(1.0, 0.0, 1.0, 1.0);
    } else {
        gl_FragColor = color;
    }
}

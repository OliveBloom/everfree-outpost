precision mediump float;

uniform sampler2D sheet_tex;
uniform sampler2D depth_tex;
uniform vec2 camera_pos;
uniform vec2 camera_size;

varying vec2 tex_coord;
varying vec3 ref_pos;
varying float ref_size_z;
varying vec3 color_;

void main(void) {
    // NB: FragCoord origin is the bottom left, not the top left.  So it works
    // for texture lookups (they use the same origin), but it needs to be
    // flipped for coordinate calculations.
    vec3 pos = vec3(gl_FragCoord.x + camera_pos.x,
                    camera_size.y - gl_FragCoord.y + camera_pos.y,
                    0.0);
    float depth = texture2D(depth_tex, gl_FragCoord.xy / camera_size).r;
    float z = floor(depth * 512.0);

    pos.y += z;
    pos.z = z;
    if (pos.z >= ref_pos.z + ref_size_z ||
            (pos.z > ref_pos.z && pos.y > ref_pos.y)) {
        discard;
    }

    vec4 base_color = texture2D(sheet_tex, tex_coord);
    if (base_color.a == 0.0) {
        discard;
    } else {
        gl_FragColor = base_color * vec4(color_, 1.0);
    }
}

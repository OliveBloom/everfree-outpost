precision mediump float;

uniform sampler2D color_tex;
uniform sampler2D meta_tex;
uniform sampler2D depth_tex;
uniform sampler2D entity_depth_tex;
uniform sampler2D light_tex;
uniform vec2 screen_size;

varying vec2 tex_coord;


// Highlight if the pixel to the left
//  1) is part of a structure
//  2) is above (higher depth) the current pixel
//  3) has z-offset 0
//  4) is not continuous with the pixel below it
float check_horiz(vec2 off, float centerDepth) {
    vec2 pos = tex_coord + off / screen_size;
    float depth = texture2D(depth_tex, pos).r;
    // #2
    if (depth < centerDepth + 8.0 / 512.0) {
        return 0.0;
    }

    vec4 color1 = texture2D(meta_tex, pos);
    if (color1.b != 1.0) {
        // #1
        return 0.0;
    }

    float baseZ = color1.r * (255.0 / 8.0 * 32.0);
    float pixelZ = depth * 512.0;

    if (pixelZ - baseZ > 0.75) {
        // #3
        return 0.0;
    }

    float neighborDepth = texture2D(depth_tex, pos + vec2(0.0, -1.0) / screen_size).r;
    float neighborDelta = (depth - neighborDepth) * 512.0;
    if (0.5 < neighborDelta && neighborDelta < 1.5) {
        // #4
        return 0.0;
    }

    float delta = depth - (centerDepth + 8.0 / 512.0);
    return clamp(delta * 512.0 / 16.0, 0.0, 1.0);
}

// Highlight if the pixel above
//  1) is part of a structure
//  2) is above (higher depth) the current pixel
//  3) has z-offset 0
float check_vert(vec2 off, float centerDepth) {
    vec2 pos = tex_coord + off / screen_size;
    float depth = texture2D(depth_tex, pos).r;
    // #2
    if (depth < centerDepth + 8.0 / 512.0) {
        return 0.0;
    }

    vec4 color1 = texture2D(meta_tex, pos);
    if (color1.b != 1.0) {
        // #1
        return 0.0;
    }

    float baseZ = color1.r * (255.0 / 8.0 * 32.0);
    float pixelZ = depth * 512.0;

    if (pixelZ - baseZ > 0.75) {
        // #3
        return 0.0;
    }

    float delta = depth - (centerDepth + 8.0 / 512.0);
    return clamp(delta * 512.0 / 16.0, 0.0, 1.0);
}

float get_highlight() {
    if (texture2D(entity_depth_tex, tex_coord).r > 0.0) {
        return 0.0;
    }

    float centerDepth = texture2D(depth_tex, tex_coord).r;
    float n = check_vert(vec2(0.0, -1.0), centerDepth);
    float s = check_vert(vec2(0.0, 1.0), centerDepth);
    float w = check_horiz(vec2(-1.0, 0.0), centerDepth);
    float e = check_horiz(vec2(1.0, 0.0), centerDepth);

    return max(max(n, s), max(w, e));
}

void main(void) {
    vec4 baseColor = texture2D(color_tex, tex_coord);

    vec4 lightColor = texture2D(light_tex, tex_coord);
    vec4 mainColor = baseColor * lightColor;

    vec4 highlightColor = vec4(0.0, 0.75, 1.0, 1.0) * lightColor.a;

    gl_FragColor = mix(mainColor, highlightColor, get_highlight());
    gl_FragColor.a = 1.0;
}

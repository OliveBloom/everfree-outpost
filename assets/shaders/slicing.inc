const float TILE_SIZE = 32.0;
const float CAVERN_MAP_RADIUS = 32.0;
const float CAVERN_MAP_SIZE = CAVERN_MAP_RADIUS * 2.0 + 4.0;
const float CAVERN_CUTOFF = 1.5 / 255.0;

int lookup(vec2 tilePos) {
    tilePos += 0.5;
    vec2 cavernPos = tilePos - sliceCenter;
    vec2 cavernTexCoord = cavernPos / CAVERN_MAP_SIZE + 0.5;
    return int(texture2D(cavernTex, cavernTexCoord).r * 255.0 + 0.5);
}

bool is_tile_inside(vec2 tilePos) {
    return lookup(tilePos) == 1;
}

float calc_val(bool inside, vec2 off, vec2 corner, float w_in, float w_out) {
    vec2 delta = off - corner;
    float d = dot(delta, delta);
        return (inside ? w_in : -w_out) / d;
}

bool calc_inside(vec2 pixelPos) {
    vec2 tilePos = floor(pixelPos / TILE_SIZE);
    if (is_tile_inside(tilePos)) {
        return true;
    } else {
        bool n = is_tile_inside(tilePos + vec2( 0.0, -1.0));
        bool s = is_tile_inside(tilePos + vec2( 0.0,  1.0));
        bool w = is_tile_inside(tilePos + vec2(-1.0,  0.0));
        bool e = is_tile_inside(tilePos + vec2( 1.0,  0.0));

        bool nw = n || w || is_tile_inside(tilePos + vec2(-1.0, -1.0));
        bool ne = n || e || is_tile_inside(tilePos + vec2( 1.0, -1.0));
        bool sw = s || w || is_tile_inside(tilePos + vec2(-1.0,  1.0));
        bool se = s || e || is_tile_inside(tilePos + vec2( 1.0,  1.0));

        vec2 finePos = pixelPos / TILE_SIZE;
        vec2 fineOffset = finePos - tilePos;
        fineOffset = (fineOffset - 0.5) * 4.0;
        fineOffset = clamp(fineOffset, -1.0, 1.0);

        // Compute magic weight factors, tuned to make the ends of curved
        // sections line up with adjacent straight sections.
        //
        //    A---*---B
        //    |   |   |
        //    |   /   |
        //    |  /    |
        //    +-/     |
        //    |       |
        //    |       |
        //    |       |
        //    C-------D
        //
        // The point marked '*' must have value exactly 0.0.  But with the
        // default formula, the balance will be 1.0 on the A side vs. 1.4 on
        // the B side (1.4 = 1/1(B) + 1/5(C) + 1/5(D)).  So we apply a
        // weighting factor, based on the number of points on the other side.
        // The weights needs to be 1.0/1.4 for a 3/1 split, and equal for a
        // 2/2 split, and otherwise we don't care.
        float num_inside =
            (nw ? 1.0 : 0.0) + (ne ? 1.0 : 0.0) +
            (sw ? 1.0 : 0.0) + (se ? 1.0 : 0.0);
        float w_in = (8.0 - num_inside) / 5.0;
        float w_out = (4.0 + num_inside) / 5.0;

        return
            calc_val(nw, fineOffset, vec2(-1.0, -1.0), w_in, w_out) +
            calc_val(ne, fineOffset, vec2( 1.0, -1.0), w_in, w_out) +
            calc_val(sw, fineOffset, vec2(-1.0,  1.0), w_in, w_out) +
            calc_val(se, fineOffset, vec2( 1.0,  1.0), w_in, w_out) > 0.0;
    }
}

bool check_slice() {
    float pixelX = cameraPos.x + gl_FragCoord.x;
    float pixelY = cameraPos.y + cameraSize.y -
        (gl_FragCoord.y - (sliceZ + 2.0) * TILE_SIZE);
    vec2 pixelPos = vec2(pixelX, pixelY);

    /*
    float dist = distance(pixelPos - vec2(0.0, sliceZ * TILE_SIZE),
        cameraPos + 0.5 * cameraSize);
    if (dist >= (CAVERN_MAP_RADIUS - 1.0) * TILE_SIZE) {
        return false;
    }
    */

    if (calc_inside(pixelPos)) {
        if (baseZ >= sliceZ + 2.0) {
            return true;
        }
    }
    return false;
}

const float TILE_SIZE = 32.0;
const float CAVERN_MAP_RADIUS = 32.0;
const float CAVERN_MAP_SIZE = CAVERN_MAP_RADIUS * 2.0 + 4.0;
const float CAVERN_CUTOFF = 1.5 / 255.0;

int sliceGetCavernTile(vec2 tilePos) {
    tilePos += 0.5;
    vec2 cavernPos = tilePos - sliceCenter;
    vec2 cavernTexCoord = cavernPos / CAVERN_MAP_SIZE + 0.5;
    return int(texture2D(cavernTex, cavernTexCoord).r * 255.0 + 0.5);
}

bool _sliceIsTileInside(vec2 tilePos) {
    return sliceGetCavernTile(tilePos) == 1;
}

vec4 _sliceCollectDists(vec2 off) {
    float d_nw = distance(off, vec2(-1.0, -1.0));
    float d_ne = distance(off, vec2( 1.0, -1.0));
    float d_sw = distance(off, vec2(-1.0,  1.0));
    float d_se = distance(off, vec2( 1.0,  1.0));
    return vec4(d_nw, d_ne, d_sw, d_se);
}

vec2 slicePixelToTile(vec2 pixelPos) {
    return floor(pixelPos / TILE_SIZE);
}

float sliceCalcInside(vec2 pixelPos) {
    vec2 tilePos = slicePixelToTile(pixelPos);
    if (_sliceIsTileInside(tilePos)) {
        return 1.0;
    } else {
        bool n = _sliceIsTileInside(tilePos + vec2( 0.0, -1.0));
        bool s = _sliceIsTileInside(tilePos + vec2( 0.0,  1.0));
        bool w = _sliceIsTileInside(tilePos + vec2(-1.0,  0.0));
        bool e = _sliceIsTileInside(tilePos + vec2( 1.0,  0.0));

        bool nw = n || w || _sliceIsTileInside(tilePos + vec2(-1.0, -1.0));
        bool ne = n || e || _sliceIsTileInside(tilePos + vec2( 1.0, -1.0));
        bool sw = s || w || _sliceIsTileInside(tilePos + vec2(-1.0,  1.0));
        bool se = s || e || _sliceIsTileInside(tilePos + vec2( 1.0,  1.0));

        vec2 finePos = pixelPos / TILE_SIZE;
        vec2 fineOffset = finePos - tilePos;
        fineOffset = (fineOffset - 0.5) * 4.0;
        fineOffset = clamp(fineOffset, -1.0, 1.0);

        vec4 inside = vec4(float(nw), float(ne), float(sw), float(se));
        float numInside = dot(inside, vec4(1.0));
        // TODO: figure out whether this is faster or slower than a texture
        // fetch.  I think it should be mostly okay since nearby fragments will
        // all take the same path...
        if (numInside == 0.0) {
            return -1.0;
        } else if (numInside == 1.0) {
            vec4 dists = _sliceCollectDists(fineOffset);
            // Find the distance to the one inside corner.
            // This value is 0 at the near corner and 2 at the two far ones
            float dist = dot(dists * inside, vec4(1.0));
            return 1.0 - dist;
        } else if (numInside == 2.0) {
            float dist;
            if (nw == se) {
                // Take the lower of the two distances toward inside vertices.
                if (nw) {
                    dist = min(distance(fineOffset, vec2(-1.0, -1.0)),
                               distance(fineOffset, vec2( 1.0,  1.0)));
                } else {
                    dist = min(distance(fineOffset, vec2(-1.0,  1.0)),
                               distance(fineOffset, vec2( 1.0, -1.0)));
                }
            } else if (nw == ne) {
                // Split along the horizontal axis
                if (nw) {
                    // Inside on top
                    dist = abs(fineOffset.y - -1.0);
                } else {
                    // Inside on bottom
                    dist = abs(fineOffset.y - 1.0);
                }
            } else if (nw == sw) {
                // Split along the vertical axis
                if (nw) {
                    // Inside on left
                    dist = abs(fineOffset.x - -1.0);
                } else {
                    // Inside on right
                    dist = abs(fineOffset.x - 1.0);
                }
            }
            return 1.0 - dist;
        } else if (numInside == 3.0) {
            vec4 dists = _sliceCollectDists(fineOffset);
            // Find the distance to the one outside corner.
            // This value is 0 at the near corner and 2 at the two far ones
            float dist = dot(dists * (1.0 - inside), vec4(1.0));
            return dist - 1.0;
        } else if (numInside == 4.0) {
            // 0.0 at the center, >1.0 at each corner
            return distance(fineOffset, vec2(0.0, 0.0));
        }
    }
}

bool sliceNearInside(vec2 pixelPos) {
    vec2 tilePos = slicePixelToTile(pixelPos);
    return
        _sliceIsTileInside(tilePos) ||
        _sliceIsTileInside(tilePos + vec2( 0.0, -1.0)) ||
        _sliceIsTileInside(tilePos + vec2( 0.0,  1.0)) ||
        _sliceIsTileInside(tilePos + vec2(-1.0,  0.0)) ||
        _sliceIsTileInside(tilePos + vec2( 1.0,  0.0)) ||
        _sliceIsTileInside(tilePos + vec2(-1.0, -1.0)) ||
        _sliceIsTileInside(tilePos + vec2( 1.0, -1.0)) ||
        _sliceIsTileInside(tilePos + vec2(-1.0,  1.0)) ||
        _sliceIsTileInside(tilePos + vec2( 1.0,  1.0));
}

vec2 sliceGetPos() {
    float pixelX = cameraPos.x + gl_FragCoord.x;
    float pixelY = cameraPos.y + cameraSize.y -
        (gl_FragCoord.y - (sliceZ + 2.0) * TILE_SIZE);
    return vec2(pixelX, pixelY);
}

bool sliceCheck() {
#ifdef SLICE_ENABLE
    vec2 pixelPos = sliceGetPos();
    return baseZ >= sliceZ + 2.0 || !sliceNearInside(pixelPos);
#else
    return false;
#endif
}

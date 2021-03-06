const float TILE_SIZE = 32.0;
const float CAVERN_MAP_SIZE = 96.0;
const float CAVERN_CUTOFF = 1.5 / 255.0;

const int BIT_INSIDE =      0x02;
const int BIT_INSIDE_NW =   0x04;
const int BIT_INSIDE_NE =   0x08;
const int BIT_INSIDE_SW =   0x10;
const int BIT_INSIDE_SE =   0x20;

int sliceGetCavernTile(vec2 tilePos) {
    tilePos += 0.5;
    vec2 cavernPos = tilePos - sliceCenter;
    vec2 cavernTexCoord = cavernPos / CAVERN_MAP_SIZE + 0.5;
    return int(texture2D(cavernTex, cavernTexCoord).r * 255.0 + 0.5);
}

bool _getBit(int x, int value) {
    int y = x / value;
    return y - (y / 2 * 2) == 1;
}

bool _sliceIsTileInside(vec2 tilePos) {
    return _getBit(sliceGetCavernTile(tilePos), 2);
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
    int bits = sliceGetCavernTile(tilePos);
    if (_getBit(bits, BIT_INSIDE)) {
        return 1.0;
    } else {
        bool nw = _getBit(bits, BIT_INSIDE_NW);
        bool ne = _getBit(bits, BIT_INSIDE_NE);
        bool sw = _getBit(bits, BIT_INSIDE_SW);
        bool se = _getBit(bits, BIT_INSIDE_SE);

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
        } else {
            return 0.0;
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

# ifndef SLICE_SIMPLIFIED
    vec2 pixelPos = sliceGetPos();
    return baseZ >= sliceZ + 2.0 || !sliceNearInside(pixelPos);
# else
    vec2 pixelPos = sliceGetPos();
    return baseZ >= sliceZ + 2.0 && sliceCalcInside(pixelPos) >= 0.0;
# endif

#else
    return false;
#endif
}

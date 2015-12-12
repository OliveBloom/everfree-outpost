var Texture = require('graphics/glutil').Texture;
var TILE_SIZE = require('data/chunk').TILE_SIZE;

/** @constructor */
function CavernMap(gl, radius) {
    this.radius = radius;
    this.gl = gl;
    this.texture = new Texture(gl);

    this.last_pos = null;
    this.invalid = false;

    // `size` needs to be a multiple of 4 due to texture stride requirements.
    // This code works as long as `radius` is a multiple of 2.
    var size = radius * 2 + 4;
    var blank = new Uint8Array(size * size);
    blank.fill(0);
    this.texture.bind();
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.LUMINANCE, size, size, 0,
            gl.LUMINANCE, gl.UNSIGNED_BYTE, blank);
    this.texture.unbind();
}
exports.CavernMap = CavernMap;

CavernMap.prototype.needsUpdate = function(pos) {
    if (this.last_pos == null || this.invalid) {
        return true;
    }

    var tile_pos = pos.divScalar(TILE_SIZE);
    if (tile_pos.x != this.last_pos.x ||
            tile_pos.y != this.last_pos.y ||
            tile_pos.z != this.last_pos.z) {
        return true;
    }
    return false;
};

CavernMap.prototype.invalidate = function() {
    this.invalid = true;
};

CavernMap.prototype.update = function(asm, pos) {
    this.last_pos = pos.divScalar(TILE_SIZE);
    this.invalid = false;

    var data = asm.floodfillCeiling(pos, this.radius);

    var size = this.radius * 2;
    var gl = this.gl;
    this.texture.bind();
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 2, 2, size, size, gl.LUMINANCE, gl.UNSIGNED_BYTE, data);
    this.texture.unbind();
};

CavernMap.prototype.getTexture = function() {
    return this.texture;
};

var Config = require('config').Config;
var AsmGraphics = require('asmlibs').AsmGraphics;
var getRendererHeapSize = require('asmlibs').getRendererHeapSize;
var getGraphicsHeapSize = require('asmlibs').getGraphicsHeapSize;
var OffscreenContext = require('graphics/canvas').OffscreenContext;
var CHUNK_SIZE = require('consts').CHUNK_SIZE;
var TILE_SIZE = require('consts').TILE_SIZE;
var LOCAL_SIZE = require('consts').LOCAL_SIZE;
var Texture = require('graphics/glutil').Texture;
var Buffer = require('graphics/glutil').Buffer;
var Framebuffer = require('graphics/glutil').Framebuffer;
var makeShaders = require('graphics/shaders').makeShaders;
var CavernMap = require('graphics/cavernmap').CavernMap;

var GlObject = require('graphics/glutil').GlObject;
var uniform = require('graphics/glutil').uniform;
var attribute = require('graphics/glutil').attribute;

var PonyAppearanceClass = require('graphics/appearance/pony').PonyAppearanceClass;

var TimeSeries = require('util/timeseries').TimeSeries;
var fstr1 = require('util/misc').fstr1;


var CHUNK_PX = CHUNK_SIZE * TILE_SIZE;

// The `now` value passed to the animation shader must be reduced to fit in a
// float.  We use the magic number 55440 for this, since it's divisible by
// every number from 1 to 12 (and most "reasonable" numbers above that).  This
// is useful because repeating animations will glitch when `now` wraps around
// unless `length / framerate` divides evenly into the modulus.
//
// Note that the shader `now` and ANIM_MODULUS are both in seconds, not ms.
var ANIM_MODULUS = 55440;

// We also need a smaller modulus for one-shot animation start times.  These
// are measured in milliseconds and must fit in a 16-bit int.  It's important
// that the one-shot modulus divides evenly into 1000 * ANIM_MODULUS, because
// the current frame time in milliseconds will be modded by 1000 * ANIM_MODULUS
// and then again by the one-shot modulus.
//
// We re-use ANIM_MODULUS as the one-shot modulus, since it obviously divides
// evenly into 1000 * ANIM_MODULUS.  This is okay as long as ANIM_MODULUS fits
// into 16 bits.
var ONESHOT_MODULUS = ANIM_MODULUS;


/** @constructor */
function RenderData(gl, asm) {
    this._asm = asm;
}

// Structure tracking

RenderData.prototype.addStructure = function(now, id, x, y, z, template_id) {
    var tx = (x / TILE_SIZE) & (LOCAL_SIZE * CHUNK_SIZE - 1);
    var ty = (y / TILE_SIZE) & (LOCAL_SIZE * CHUNK_SIZE - 1);
    var tz = (z / TILE_SIZE) & (LOCAL_SIZE * CHUNK_SIZE - 1);

    var oneshot_start = now % ONESHOT_MODULUS;
    if (oneshot_start < 0) {
        oneshot_start += ONESHOT_MODULUS;
    }
    this._asm.structureAppear(id, tx, ty, tz, template_id, oneshot_start);
};

RenderData.prototype.removeStructure = function(id) {
    this._asm.structureGone(id);
};

RenderData.prototype.replaceStructure = function(now, id, template_id) {
    var oneshot_start = now % ONESHOT_MODULUS;
    if (oneshot_start < 0) {
        oneshot_start += ONESHOT_MODULUS;
    }
    this._asm.structureReplace(id, template_id, oneshot_start);
};


/** @constructor */
function Renderer(gl, assets, asm) {
    this.gl = gl;
    this.data = new RenderData(gl, asm);
}
exports.Renderer = Renderer;

Renderer.prototype.addStructure = function(now, id, x, y, z, template_id) {
    return this.data.addStructure(now, id, x, y, z, template_id);
};

Renderer.prototype.removeStructure = function(id) {
    return this.data.removeStructure(id);
};

Renderer.prototype.replaceStructure = function(now, id, template_id) {
    return this.data.replaceStructure(now, id, template_id);
};

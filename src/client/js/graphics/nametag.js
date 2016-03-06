var FontMetrics = require('data/fontmetrics').FontMetrics;
var OffscreenContext = require('graphics/canvas').OffscreenContext;
var StringCache = require('util/stringcache').StringCache;


var NAME_WIDTH = 96;
var NAME_HEIGHT = 12;
var NAME_BUFFER_WIDTH = 512;
var NAME_BUFFER_HEIGHT = 512;
var NAME_BUFFER_COUNT_X = (NAME_BUFFER_WIDTH / NAME_WIDTH)|0;
var NAME_BUFFER_COUNT_Y = (NAME_BUFFER_HEIGHT / NAME_HEIGHT)|0;
var NAME_BUFFER_COUNT = NAME_BUFFER_COUNT_X * NAME_BUFFER_COUNT_Y;

exports.NAME_WIDTH = NAME_WIDTH;
exports.NAME_HEIGHT = NAME_HEIGHT;
exports.NAME_BUFFER_WIDTH = NAME_BUFFER_WIDTH;
exports.NAME_BUFFER_HEIGHT = NAME_BUFFER_HEIGHT;


/** @constructor */
function NameBuffer(assets) {
    this.ctx = new OffscreenContext(NAME_BUFFER_WIDTH, NAME_BUFFER_HEIGHT);
    this.cache = new StringCache(NAME_BUFFER_COUNT);

    this.font_img = assets['fonts'];
}
exports.NameBuffer = NameBuffer;

NameBuffer.prototype._draw = function(s, idx) {
    var x = NAME_WIDTH * (idx % NAME_BUFFER_COUNT_X);
    var y = NAME_HEIGHT * ((idx / NAME_BUFFER_COUNT_X)|0);
    var ctx = this.ctx;

    var fm = FontMetrics.by_name['name'];
    var str_width = fm.measureWidth(s);
    var offset_x = Math.floor((NAME_WIDTH - str_width) / 2);

    ctx.save();

    ctx.clearRect(x, y, NAME_WIDTH, NAME_HEIGHT);
    ctx.rect(x, y, NAME_WIDTH, NAME_HEIGHT);
    ctx.clip();
    var img = this.font_img;
    fm.drawString(s, function(sx, sy, w, h, dx, dy) {
        ctx.drawImage(img,
                sx, sy, w, h,
                x + offset_x + dx, y + dy, w, h);
    });

    ctx.restore();
};

NameBuffer.prototype.offset = function(s) {
    var idx = this.cache.get(s);
    var created = false;
    if (idx == null) {
        idx = this.cache.put(s);
        this._draw(s, idx);
        created = true;
    }

    var x = NAME_WIDTH * (idx % NAME_BUFFER_COUNT_X);
    var y = NAME_HEIGHT * ((idx / NAME_BUFFER_COUNT_X)|0);
    return { x: x, y: y, created: created };
};

NameBuffer.prototype.image = function() {
    return this.ctx.canvas;
};

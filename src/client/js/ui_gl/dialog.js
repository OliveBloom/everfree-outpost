var Config = require('config').Config;
var ItemDef = require('data/items').ItemDef;
var FontMetrics = require('data/fontmetrics').FontMetrics;
var W = require('ui_gl/widget');


/** @constructor */
function DialogBody() {
    W.Widget.call(this);
    this.layout = new PaddedPaneLayout(6, 6, 6, 6);

    this.content = null;
}
DialogBody.prototype = Object.create(W.Widget.prototype);
DialogBody.prototype.constructor = DialogBody;
exports.DialogBody = DialogBody;

DialogBody.prototype.setContent = function(content) {
    if (content === this.content) {
        return;
    }

    if (this.content != null) {
        this.removeChild(this.content);
    }
    this.content = content;
    if (this.content != null) {
        this.addChild(this.content);
    }
};

var DIALOG_BORDER_IMG_SIZE = {
    t: 8,
    b: 8,
    l: 11,
    r: 10,
};

DialogBody.prototype.render = function(buf, x, y) {
    if (this.content == null) {
        return;
    }

    var w = this._width;
    var h = this._height;
    var B = DIALOG_BORDER_IMG_SIZE;

    buf.drawUI('dialog-body-center',
            x + B.l, y + B.t,       w - B.l - B.r, h - B.t - B.b);

    buf.drawUI('dialog-body-n',
            x + B.l, y,             w - B.l - B.r, B.t);
    buf.drawUI('dialog-body-s',
            x + B.l, y + h - B.b,   w - B.l - B.r, B.b);
    buf.drawUI('dialog-body-w',
            x, y + B.t,             B.l, h - B.t - B.b);
    buf.drawUI('dialog-body-e',
            x + w - B.r, y + B.t,   B.r, h - B.t - B.b);

    buf.drawUI('dialog-body-nw',
            x, y,                           B.l, B.t);
    buf.drawUI('dialog-body-ne',
            x + w - B.r, y,                 B.r, B.t);
    buf.drawUI('dialog-body-se',
            x + w - B.r, y + h - B.b,       B.r, B.b);
    buf.drawUI('dialog-body-sw',
            x, y + h - B.b,                 B.l, B.b);
};


var DIALOG_TITLE_HEIGHT = 30;
var DIALOG_TITLE_SIDE_WIDTH = 17;

/** @constructor */
function DialogTitle() {
    W.Widget.call(this);
    this.layout = new FixedSizeLayout(0, DIALOG_TITLE_HEIGHT);

    this.text = '';
}
DialogTitle.prototype = Object.create(W.Widget.prototype);
DialogTitle.prototype.constructor = DialogTitle;
exports.DialogTitle = DialogTitle;

DialogTitle.prototype.setText = function(text) {
    this.text = text;
    this.damage();
};

DialogTitle.prototype.calcSize = function(w, h) {
    this.layout.w = w;
    // Ignore `h`.  Height is always DIALOG_TITLE_HEIGHT.
};

DialogTitle.prototype.render = function(buf, x, y) {
    buf.drawUI('dialog-title-left', x, y);
    buf.drawUI('dialog-title-right', x + this._width - DIALOG_TITLE_SIDE_WIDTH, y);
    buf.drawUI('dialog-title-center',
            x + DIALOG_TITLE_SIDE_WIDTH, y,
            this._width - 2 * DIALOG_TITLE_SIDE_WIDTH, null);

    var fm = FontMetrics.by_name['name'];
    var text_width = fm.measureWidth(this.text);
    var out_x = x + Math.floor((this._width - text_width) / 2);
    var out_y = y + Math.floor((this._height - fm.height) / 2);
    fm.drawString(this.text, function(sx, sy, w, h, dx, dy) {
        buf.drawChar(sx, sy, w, h, out_x + dx, out_y + dy);
    });
};


// There are 7 pixels on either side of the title that are too far up for a
// spacer to connect to.  There are 8 pixels on either side of the spacer
// graphic that are actually transparent, and don't count.
var DIALOG_SPACER_INSET = 7 - 8;
var DIALOG_SPACER_WIDTH = 20;

/** @constructor */
function DialogGL() {
    W.Widget.call(this);

    this.title = new DialogTitle();
    this.body = new DialogBody();

    this.addChild(this.title);
    this.addChild(this.body);
}
DialogGL.prototype = Object.create(W.Widget.prototype);
DialogGL.prototype.constructor = DialogGL;
exports.DialogGL = DialogGL;

DialogGL.prototype.setContent = function(content) {
    this.body.setContent(content);
};

DialogGL.prototype.setTitle = function(text) {
    this.title.setText(text);
};

DialogGL.prototype.runLayout = function() {
    this.body.runLayout();
    this.body._width += 60; // HACK
    this.title.calcSize(this.body._width, 0);
    this.title.runLayout();

    this.title._x = 0;
    this.title._y = 0;

    this.body._x = 0;
    this.body._y = this.title._height + 3;

    this._width = this.body._width;
    this._height = this.title._height + this.body._height + 3;
};

DialogGL.prototype.render = function(buf, x, y) {
    var spacer_max_width = this._width - 2 * DIALOG_SPACER_INSET;
    var spacer_count = Math.floor(spacer_max_width / DIALOG_SPACER_WIDTH);
    var spacer_width = spacer_count * DIALOG_SPACER_WIDTH;

    var spacer_x = Math.floor((this._width - spacer_width) / 2);

    buf.drawUI('dialog-spacer',
            x + spacer_x, y + this.title._height - 2,
            spacer_width, null);
};

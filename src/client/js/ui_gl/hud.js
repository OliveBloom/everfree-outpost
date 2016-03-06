var FontMetrics = require('data/fontmetrics').FontMetrics;
var W = require('ui_gl/widget');

var Hotbar = require('ui_gl/hotbar').Hotbar;
var TestWidget = require('ui_gl/dialog').DialogGL;


/** @constructor */
function GameUI() {
    W.Widget.call(this);
    this.hotbar = new Hotbar();
    this.fps = new FPSDisplay();

    this.addChild(this.hotbar);
    this.addChild(this.fps);

    this.test = new TestWidget();
    this.test.setContent(new Hotbar());
    this.test.setTitle('hello');
    this.addChild(this.test);
}
GameUI.prototype = Object.create(W.Widget.prototype);
GameUI.prototype.constructor = GameUI;
exports.GameUI = GameUI;

GameUI.prototype.calcSize = function(w, h) {
    if (w != this._width || h != this._height) {
        this._width = w;
        this._height = h;
        this.runLayout();
    }
};

GameUI.prototype.runLayout = function() {
    this._x = 0;
    this._y = 0;

    this.hotbar.runLayout();
    this.hotbar._x = 1;
    this.hotbar._y = 1;

    this.fps.runLayout();
    this.fps._x = this._width - 1;
    this.fps._y = 1;

    this.test.runLayout();
    this.test._x = 100;
    this.test._y = 50;
};


/** @constructor */
function FPSDisplay() {
    W.Widget.call(this);
    this.layout = new W.FixedSizeLayout(0, 0);
    this.visible = false;
    this.timer = null;
}
FPSDisplay.prototype = Object.create(W.Widget.prototype);
FPSDisplay.prototype.constructor = FPSDisplay;

FPSDisplay.prototype.render = function(buf, x, y) {
    if (!this.visible) {
        return;
    }
    var s = window.DEBUG._fps + ' FPS';
    var fm = FontMetrics.by_name['name'];
    x -= fm.measureWidth(s);
    fm.drawString(s, function(sx, sy, w, h, dx, dy) {
        buf.drawChar(sx, sy, w, h, x + dx, y + dy);
    });
};

FPSDisplay.prototype.show = function() {
    if (this.visible) {
        return;
    }
    var this_ = this;
    this.timer = window.setInterval(function() { this_.damage(); }, 1000);
    this.visible = true;
    this.damage();
};

FPSDisplay.prototype.hide = function() {
    if (!this.visible) {
        return;
    }
    window.clearInterval(this.timer);
    this.timer = null;
    this.visible = false;
    this.damage();
};

FPSDisplay.prototype.toggle = function(visible) {
    if (visible) {
        this.show();
    } else {
        this.hide();
    }
};

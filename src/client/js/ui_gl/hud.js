var FontMetrics = require('data/fontmetrics').FontMetrics;
var W = require('ui_gl/widget');
var old_widget = require('ui/widget');

var Hotbar = require('ui_gl/hotbar').Hotbar;
var DialogGL = require('ui_gl/dialog').DialogGL;
var TestWidget = require('ui_gl/inventory').InventoryGrid;


/** @constructor */
function GameUI(keyboard) {
    W.Widget.call(this);

    this.keyboard = keyboard;

    this.hotbar = new Hotbar();
    this.fps = new FPSDisplay();
    this.dialog = new DialogGL();

    this.addChild(this.hotbar);
    this.addChild(this.fps);
    // dialog is added/removed depending on visibility
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

GameUI.prototype.showDialog = function(content, title) {
    if (this.dialog.hasContent()) {
        this.hideDialog();
    }
    this.dialog.setContent(content);
    this.dialog.setTitle(title || '');
    this.addChild(this.dialog);

    var this_ = this;
    this.keyboard.pushHandler(function(down, evt) {
        var widget_evt = new old_widget.WidgetKeyEvent(down, evt);
        if (down) {
            var handled = this_.dialog.onKey(widget_evt);
            return handled && !widget_evt.useDefault;
        } else {
            return true;
        }
    });
};

GameUI.prototype.hideDialog = function() {
    if (this.dialog.hasContent()) {
        this.dialog.setContent(null);
        this.removeChild(this.dialog);
    }

    this.keyboard.popHandler();
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

    if (this.dialog.hasContent()) {
        this.dialog.runLayout();
        this.dialog._x = ((this._width - this.dialog._width) / 2)|0;
        this.dialog._y = ((this._height - this.dialog._height) / 2)|0;
    }
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

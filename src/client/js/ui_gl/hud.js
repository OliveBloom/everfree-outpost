var FontMetrics = require('data/fontmetrics').FontMetrics;
var W = require('ui_gl/widget');
var old_widget = require('ui/widget');

var Hotbar = require('ui_gl/hotbar').Hotbar;
var DialogGL = require('ui_gl/dialog').DialogGL;
var ItemDisplay = require('ui_gl/inventory').ItemDisplay;


/** @constructor */
function GameUI(keyboard) {
    W.Widget.call(this);

    this.keyboard = keyboard;

    this.hotbar = new Hotbar();
    this.fps = new FPSDisplay();
    this.dialog = new DialogGL();
    this.drag_icon = new ItemDisplay()

    this.addChild(this.hotbar);
    this.addChild(this.fps);
    this.addChild(this.dialog);
    this.addChild(this.drag_icon);

    this.dialog.setHidden(true);
    this.drag_icon.setHidden(true);
    this.drag_icon.setDynamic(true);

    this.drag_active = false;

    var this_ = this;

    this.addListener('dragstart', function(type, data, x, y) {
        if (type == 'inv_items') {
            this_._dragStart(data, x, y);
        }
    });

    this.addListener('drag', function(x, y) {
        if (this_.drag_active) {
            this_._dragMove(x, y);
        }
    });

    this.addListener('dragend', function() {
        if (this_.drag_active) {
            this_._dragEnd();
        }
    });

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
    this.dialog.setHidden(false);

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
        this.dialog.setHidden(true);
    }

    this.keyboard.popHandler();
};

GameUI.prototype._dragStart = function(data, x, y) {
    var icon = this.drag_icon;
    icon.setItem(data.item_id);
    icon.setQuantity(data.quantity);
    icon.setHidden(false);
    this.drag_active = true;
    this._dragMove(x, y);
};

GameUI.prototype._dragMove = function(x, y) {
    var icon = this.drag_icon;
    icon._x = x;
    icon._y = y;
    icon.damage();
};

GameUI.prototype._dragEnd = function() {
    var icon = this.drag_icon;
    icon.setHidden(true);
    this.drag_active = false;
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

    this.dialog.runLayout();
    this.dialog._x = ((this._width - this.dialog._width) / 2)|0;
    this.dialog._y = ((this._height - this.dialog._height) / 2)|0;
};


/** @constructor */
function FPSDisplay() {
    W.Widget.call(this);
    this.layout = new W.FixedSizeLayout(0, 0);
    this.setDynamic(true);

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

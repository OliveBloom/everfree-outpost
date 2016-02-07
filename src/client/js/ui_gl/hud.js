var W = require('ui_gl/widget');

var Hotbar = require('ui_gl/hotbar').Hotbar;


/** @constructor */
function GameUI() {
    W.Widget.call(this);
    this.hotbar = new Hotbar();

    this.addChild(this.hotbar);
}
GameUI.prototype = Object.create(W.Widget.prototype);
GameUI.prototype.constructor = GameUI;
exports.GameUI = GameUI;

GameUI.prototype.calcSize = function(w, h) {
    this._width = w;
    this._height = h;
};

GameUI.prototype.runLayout = function() {
    this._x = 0;
    this._y = 0;

    this.hotbar.runLayout();
    this.hotbar._x = 1;
    this.hotbar._y = 1;
};

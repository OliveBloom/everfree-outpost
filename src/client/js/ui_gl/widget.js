/** @constructor */
function Widget() {
    this.owner = null;
    this.children = [];
    this.layout = null;
    this._x = null;
    this._y = null;
    this._width = null;
    this._height = null;
}
exports.Widget = Widget;

Widget.prototype.addChild = function(w) {
    if (w.owner != null) {
        w.owner.removeChild(w);
    }
    w.owner = this;
    this.children.push(w);
};

Widget.prototype.removeChild = function(w) {
    if (w.owner !== this) {
        return;
    }
    w.owner = null;
    var index = this.children.indexOf(w);
    console.assert(index != -1, "child widget not found in this.children");
    this.children.splice(index, 1);
};

Widget.prototype.runLayout = function() {
    for (var i = 0; i < this.children.length; ++i) {
        this.children[i].runLayout();
    }
    this.layout.runLayout(this, this.children);
};

Widget.prototype.damage = function() {
    // TODO
};

Widget.prototype.calcSize = function(w, h) {
    // TODO: not sure what a reasonable default is here
};


/** @constructor */
function FixedSizeLayout(w, h) {
    this.w = w;
    this.h = h;
}
exports.FixedSizeLayout = FixedSizeLayout;

FixedSizeLayout.prototype.runLayout = function(owner, children) {
    console.assert(children.length == 0, "FixedSizeLayout does not support children");
    owner._width = this.w;
    owner._height = this.h;
};


/** @constructor */
function ColumnLayout(spacing) {
    this.spacing = spacing;
}
exports.ColumnLayout = ColumnLayout;

ColumnLayout.prototype.runLayout = function(owner, children) {
    var width = 0;
    for (var i = 0; i < children.length; ++i) {
        width = Math.max(width, children[i]._width);
    }

    var y = 0;
    for (var i = 0; i < children.length; ++i) {
        var c = children[i];
        c._x = (width - c._width) / 2;
        c._y = y;
        y += c._height + this.spacing;
    }
    if (children.length > 0) {
        y -= this.spacing;
    }

    owner._width = width;
    owner._height = y;
};


/** @constructor */
function Spacer(w, h) {
    Widget.call(this);
    this.layout = new FixedSizeLayout(w, h);
};
Spacer.prototype = Object.create(Widget.prototype);
Spacer.prototype.constructor = Spacer;
exports.Spacer = Spacer;



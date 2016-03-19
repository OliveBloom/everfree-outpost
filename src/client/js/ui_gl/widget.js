
var FLAG_DAMAGED =                  0x0001;
var FLAG_STATIC_CHILD_DAMAGED =     0x0002;
var FLAG_DYNAMIC_CHILD_DAMAGED =    0x0004;
var FLAG_DYNAMIC =                  0x0008
var FLAG_LAYOUT_DAMAGED =           0x0010;

var MASK_ANY_DAMAGED =
    FLAG_DAMAGED |
    FLAG_STATIC_CHILD_DAMAGED |
    FLAG_DYNAMIC_CHILD_DAMAGED;

exports.FLAG_DAMAGED = FLAG_DAMAGED;
exports.FLAG_STATIC_CHILD_DAMAGED = FLAG_STATIC_CHILD_DAMAGED;
exports.FLAG_DYNAMIC_CHILD_DAMAGED = FLAG_DYNAMIC_CHILD_DAMAGED;
exports.FLAG_DYNAMIC = FLAG_DYNAMIC;
exports.FLAG_LAYOUT_DAMAGED = FLAG_LAYOUT_DAMAGED;
exports.MASK_ANY_DAMAGED = MASK_ANY_DAMAGED;

/** @constructor */
function Widget() {
    this.owner = null;
    this.children = [];
    this.layout = null;
    this._x = null;
    this._y = null;
    this._width = null;
    this._height = null;
    this._flags = 0;
    this._listeners = {};
}
exports.Widget = Widget;

Widget.prototype.addChild = function(w) {
    if (w.owner != null) {
        w.owner.removeChild(w);
    }
    w.owner = this;
    this.children.push(w);
    this.damageLayout();
};

Widget.prototype.removeChild = function(w) {
    if (w.owner !== this) {
        return;
    }
    w.owner = null;
    var index = this.children.indexOf(w);
    console.assert(index != -1, "child widget not found in this.children");
    this.children.splice(index, 1);
    this.damageLayout();
};

Widget.prototype.setDynamic = function(set) {
    var cur = !!(this._flags & FLAG_DYNAMIC);
    if (cur == set) {
        return;
    }

    if (set) {
        this._flags |= FLAG_DYNAMIC;
    } else {
        this._flags &= FLAG_DYNAMIC;
    }
    // Force rebuilding of *both* buffers.  Otherwise we may get multiple
    // copies of the same widget, one static and one dynamic.
    this._damageRecursive(FLAG_STATIC_CHILD_DAMAGED);
    this._damageRecursive(FLAG_DYNAMIC_CHILD_DAMAGED);
};

Widget.prototype.addListener = function(name, func) {
    var l = this._listeners[name];
    if (l == null) {
        this._listeners[name] = [func];
    } else {
        // Avoid duplicate listeners, like the real addEventListener does
        if (l.indexOf(func) != -1) {
            l.append(func);
        }
    }
};

Widget.prototype.removeListener = function(name, func) {
    var l = this._listeners[name];
    if (l == null) {
        return;
    }

    var index = l.indexOf(func);
    if (index == -1) {
        return;
    }
    l.splice(index, 1);
    if (l.length == 0) {
        this._listeners[name] = null;
    }
};

Widget.prototype.hasListener = function(name) {
    return this._listeners[name] != null;
}

Widget.prototype.dispatch = function(name /* varargs */) {
    var l = this._listeners[name];
    if (l == null) {
        return;
    }

    var args = Array.prototype.slice.call(arguments, 1);
    var result = undefined;
    for (var i = 0; i < l.length; ++i) {
        result = l[i].apply(this, args);
    }
    return result;
};

Widget.prototype.runLayout = function() {
    for (var i = 0; i < this.children.length; ++i) {
        this.children[i].runLayout();
    }
    this.layout.runLayout(this, this.children);
};

Widget.prototype.damage = function() {
    if (!(this._flags & FLAG_DAMAGED)) {
        this._flags |= FLAG_DAMAGED;
        if (this._flags & FLAG_DYNAMIC) {
            this._damageRecursive(FLAG_DYNAMIC_CHILD_DAMAGED);
        } else {
            this._damageRecursive(FLAG_STATIC_CHILD_DAMAGED);
        }
    }
};

Widget.prototype._damageRecursive = function(flag) {
    if (!(this._flags & flag)) {
        this._flags |= flag;
        if (this.owner != null) {
            this.owner._damageRecursive(flag);
        }
    }
};

Widget.prototype.damageLayout = function() {
    this._damageRecursive(FLAG_LAYOUT_DAMAGED);
};

Widget.prototype.render = function(buffers) {
};

Widget.prototype.calcSize = function(w, h) {
    // TODO: not sure what a reasonable default is here
};

Widget.prototype.onKey = function(evt) {
    return false;
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
function PaddedPaneLayout(t, b, l, r) {
    this.t = t;
    this.b = b;
    this.l = l;
    this.r = r;
}
exports.PaddedPaneLayout = PaddedPaneLayout;

PaddedPaneLayout.prototype.runLayout = function(owner, children) {
    console.assert(children.length < 2, "PaddedPaneLayout requires exactly 0 or 1 children");

    if (children.length == 1) {
        var c = children[0];
        c._x = this.l;
        c._y = this.t;
        owner._width = c._width + this.l + this.r;
        owner._height = c._height + this.t + this.b;
    } else {
        owner._width = this.l + this.r;
        owner._height = this.t + this.b;
    }
}


/** @constructor */
function Spacer(w, h) {
    Widget.call(this);
    this.layout = new FixedSizeLayout(w, h);
};
Spacer.prototype = Object.create(Widget.prototype);
Spacer.prototype.constructor = Spacer;
exports.Spacer = Spacer;



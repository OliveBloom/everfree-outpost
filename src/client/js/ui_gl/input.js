
function hitTest(rx, ry, w) {
    return (rx >= 0 && rx < w._width && ry >= 0 && ry < w._height);
}

function MouseStackEntry(widget, x, y) {
    this.widget = widget;
    this.abs_x = x;
    this.abs_y = y;
}

MouseStackEntry.prototype.contains = function(x, y) {
    return hitTest(x - this.abs_x, y - this.abs_y, this.widget);
};


function UIInput(root) {
    this.root = root;

    this.mouse_stack = [];
}
exports.UIInput = UIInput;

UIInput.prototype.handleKeyDown = function(evt) {
    if (this.root.handleKeyDown) {
        return this.root.handleKeyDown(evt);
    }
};

UIInput.prototype.handleKeyUp = function(evt) {
    if (this.root.handleKeyUp) {
        return this.root.handleKeyUp(evt);
    }
};

UIInput.prototype.handleMouseOver = function(evt) {
    this._updateMouseStack(evt.x, evt.y, evt);
    return true;
};

UIInput.prototype.handleMouseOut = function(evt) {
    this._updateMouseStack(-1, -1, evt);
    return true;
};

UIInput.prototype.handleMouseMove = function(evt) {
    this._updateMouseStack(evt.x, evt.y, evt);
    return true;
};

UIInput.prototype.handleMouseDown = function(evt) {
    if (this.mouse_stack.length == 0) {
        return false;
    }

    var s = this.mouse_stack;
    for (var i = s.length - 1; i >= 0; --i) {
        if (s[i].widget.handleMouseDown) {
            return s[i].widget.handleMouseDown(evt);
        }
    }
};

UIInput.prototype.handleMouseUp = function(evt) {
    if (this.mouse_stack.length == 0) {
        return false;
    }

    var s = this.mouse_stack;
    for (var i = s.length - 1; i >= 0; --i) {
        if (s[i].widget.handleMouseUp) {
            return s[i].widget.handleMouseUp(evt);
        }
    }
};

UIInput.prototype._updateMouseStack = function(x, y, evt) {
    var s = this.mouse_stack;

    // Pop elements that have been exited
    while (s.length > 0 && !s[s.length - 1].contains(x, y)) {
        var entry = s.pop();
        console.log('exit', entry.widget.constructor.name);
        if (entry.widget.handleMouseOut) {
            entry.widget.handleMouseOut(evt);
        }
    }

    // Push elements that have been entered

    // Absolute coordinates of the top widget on the stack.
    var ax;
    var ay;

    // First, try the root element (if necessary)
    if (s.length == 0) {
        ax = this.root._x;
        ay = this.root._y;
        if (hitTest(x - ax, y - ay, this.root)) {
            console.log('enter', this.root.constructor.name);
            if (this.root.handleMouseOver) {
                this.root.handleMouseOver(evt);
            }
            s.push(new MouseStackEntry(this.root, this.root._x, this.root._y));
        } else {
            // It's not inside anything at all
            return;
        }
    } else {
        ax = s[s.length - 1].abs_x;
        ay = s[s.length - 1].abs_y;
    }

    // Next, loop over children
    var keep_going = true;
    while (keep_going) {
        keep_going = false;

        var p = s[s.length - 1].widget;
        for (var i = 0; i < p.children.length; ++i) {
            var c = p.children[i];
            if (hitTest(x - (ax + c._x), y - (ay + c._y), c)) {
                console.log('enter', c.constructor.name);
                if (c.handleMouseOver) {
                    c.handleMouseOver(evt);
                }
                ax += c._x;
                ay += c._y;
                s.push(new MouseStackEntry(c, ax, ay));
                keep_going = true;
                break;
            }
        }
    }
};

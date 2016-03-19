var W = require('ui_gl/widget');

function hitTest(rx, ry, w) {
    if (w._flags & W.FLAG_HIDDEN) {
        return false;
    }
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

    this.drag_source = null;
    this.drag_type = null;
    this.drag_data = null;
    this.drag_x = -1;
    this.drag_y = -1;
    // `true` once the dragging actually begins (after the initial mousedown, a
    // mousemove or mouseup event).
    this.drag_active = false;
}
exports.UIInput = UIInput;


UIInput.prototype.startDrag = function(source, evt, type, data) {
    this.drag_source = source;
    this.drag_type = type;
    this.drag_data = data;
    this.drag_active = false;

    this.drag_x = evt.x;
    this.drag_y = evt.y;
    this.root.dispatch('dragstart', this.drag_type, this.drag_data,
            this.drag_x, this.drag_y, this);

    this._updateDragCursor();
};

UIInput.prototype.dragcancel = function() {
    if (this.drag_source) {
        this.drag_source.dispatch('dragcancel', this.drag_type, this.drag_data, this);
    }
    this._endDrag();
};

UIInput.prototype._finishDrag = function() {
    var found_target = false;
    var s = this.mouse_stack;
    for (var i = s.length - 1; i >= 0; --i) {
        var w = s[i].widget;
        // `w` only needs 'dropcheck' to stop the search, but it needs both
        // 'drop' and a passing 'dropcheck' to be considered a valid target.
        if (w.hasListener('dropcheck')) {
            if (w.hasListener('drop') && 
                    w.dispatch('dropcheck', this.drag_type, this.drag_data, this)) {
                found_target = true;
                w.dispatch('drop', this.drag_type, this.drag_data, this);
            }
            break;
        }
    }

    if (found_target) {
        this._endDrag();
    } else {
        // Let the source know the drop failed.
        this.dragcancel();
    }
};

UIInput.prototype._updateDragCursor = function() {
    var drop_ok = this._dispatchTopmost('dropcheck', this.drag_type, this.drag_data, this);
    document.body.style.cursor = drop_ok ? 'grabbing' : 'not-allowed';
};

UIInput.prototype._endDrag = function() {
    this.drag_type = null;
    this.drag_data = null;
    this.drag_active = false;

    document.body.style.cursor = 'auto';

    this.root.dispatch('dragend', this);
};

UIInput.prototype._dragging = function() {
    return this.drag_type != null;
};


UIInput.prototype.handleKeyDown = function(evt) {
    if (this._dragging()) {
        if (evt.keyName() == 'cancel') {
            this.dragcancel();
        }
        return true;
    }

    this.root.dispatch('keydown', evt, this);
};

UIInput.prototype.handleKeyUp = function(evt) {
    if (this._dragging()) {
        return true;
    }

    this.root.dispatch('keyup', evt, this);
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

    if (this._dragging()) {
        this.drag_active = true;
        if (evt.x != this.drag_x || evt.y != this.drag_y) {
            this.drag_x = evt.x;
            this.drag_y = evt.y;
            this.root.dispatch('drag', this.drag_x, this.drag_y, this);
        }
    }

    return true;
};

UIInput.prototype.handleMouseDown = function(evt) {
    if (this._dragging()) {
        return true;
    }

    if (this.mouse_stack.length == 0) {
        return false;
    }

    this._dispatchTopmost('mousedown', evt, this);
};

UIInput.prototype.handleMouseUp = function(evt) {
    if (this._dragging()) {
        if (!this.drag_active) {
            this.drag_active = true;
        } else {
            this._finishDrag();
        }
    }

    if (this.mouse_stack.length == 0) {
        return false;
    }

    this._dispatchTopmost('mouseup', evt, this);
};

UIInput.prototype._dispatchTopmost = function(name /* varargs */) {
    var args = Array.prototype.slice.call(arguments);
    var s = this.mouse_stack;
    for (var i = s.length - 1; i >= 0; --i) {
        var w = s[i].widget;
        if (w.hasListener(name)) {
            return w.dispatch.apply(w, args);
        }
    }
};


UIInput.prototype._fireMouseOut = function(target, evt) {
    if (target.handleMouseOut) {
        target.handleMouseOut(evt, this);
    }
};

UIInput.prototype._fireMouseOver = function(target, evt) {
    if (target.handleMouseOver) {
        target.handleMouseOver(evt, this);
    }
};

UIInput.prototype._updateMouseStack = function(x, y, evt) {
    var s = this.mouse_stack;

    // Check if the stack actually changed.  If it did, we may need to update
    // the drag/drop cursor.
    //
    // Note that mouseover/mouseout events still fire during drap/drop.  This
    // seems like a good idea at least for now.
    var changed = false;

    // Pop elements that have been exited
    while (s.length > 0 && !s[s.length - 1].contains(x, y)) {
        var entry = s.pop();
        entry.widget.dispatch('mouseout', evt, this);
        changed = true;
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
            this.root.dispatch('mouseover', evt, this);
            s.push(new MouseStackEntry(this.root, this.root._x, this.root._y));
            changed = true;
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
                c.dispatch('mouseover', evt, this);
                ax += c._x;
                ay += c._y;
                s.push(new MouseStackEntry(c, ax, ay));
                changed = true;
                keep_going = true;
                break;
            }
        }
    }

    // Do drag/drop cursor update.
    if (this._dragging() && changed) {
        this._updateDragCursor();
    }
};

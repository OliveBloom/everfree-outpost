var Config = require('config').Config;


/** @constructor */
function InputEvent() {
    this._forward = false;
    this._default = false;
}

InputEvent.prototype.forward = function() {
    this._forward = true;
};

InputEvent.prototype.requestDefault = function() {
    this._default = true;
};

InputEvent.prototype._reset = function() {
    this._forward = false;
    this._default = false;
};


/** @constructor */
function KeyEvent(evt) {
    this.raw = evt;
    this.shift = evt.shiftKey;
}
KeyEvent.prototype = Object.create(InputEvent.prototype);
KeyEvent.prototype.constructor = KeyEvent;

KeyEvent.prototype.shouldIgnore = function() {
    return this.raw.ctrlKey || this.raw.altKey || this.raw.metaKey;
};

KeyEvent.prototype.keyName = function() {
    if (this.shouldIgnore()) {
        return null;
    }
    return Config.keybindings.get()[this.raw.keyCode];
};

KeyEvent.prototype.chatKeyName = function() {
    if (this.shouldIgnore()) {
        return null;
    }
    return Config.keybindings.get()[this.raw.keyCode];
};

KeyEvent.prototype.uiKeyName = function() {
    if (this.shouldIgnore()) {
        return null;
    }
    return Config.keybindings.get()[this.raw.keyCode];
};


/** @constructor */
function MouseEvent(evt, x, y) {
    this.raw = evt;
    this.button = evt.button;
    this.shift = evt.shiftKey;

    var scale = +document.body.dataset['worldScale'];
    this.x = (evt.pageX / scale)|0;
    this.y = (evt.pageY / scale)|0;
}
MouseEvent.prototype = Object.create(InputEvent.prototype);
MouseEvent.prototype.constructor = MouseEvent;


function alwaysStopKey(evt) {
    // TODO: do this for now until <input type=text> handling is figured out
    // The old keyboard.js "always stop" code is smarter anyhow.
    return false;

    // Allow Ctrl + anything
    if (evt.ctrlKey) {
        return false;
    }
    // Allow F5-F12
    if (evt.keyCode >= 111 + 5 && evt.keyCode <= 111 + 12) {
        return false;
    }

    // Stop all other events.
    return true;
}

function alwaysStopMouse(evt) {
    return true;
}


/** @constructor */
function Input() {
    this.handlers = [];

    this.last_key = null;
}
exports.Input = Input;

Input.prototype.attach = function(doc) {
    var this_ = this;

    doc.addEventListener('keydown', function(evt) {
        this_.handleKeyDown(evt);
    });
    doc.addEventListener('keyup', function(evt) {
        this_.handleKeyUp(evt);
    });

    doc.addEventListener('mouseover', function(evt) {
        this_.handleMouseOver(evt);
    });
    doc.addEventListener('mouseout', function(evt) {
        this_.handleMouseOut(evt);
    });
    doc.addEventListener('mousemove', function(evt) {
        this_.handleMouseMove(evt);
    });
    doc.addEventListener('mousedown', function(evt) {
        this_.handleMouseDown(evt);
    });
    doc.addEventListener('mouseup', function(evt) {
        this_.handleMouseUp(evt);
    });

    doc.addEventListener('contextmenu', function(evt) {
        evt.preventDefault();
    });
};

Input.prototype._cleanup = function(evt, ret) {
    if (ret && !evt._default) {
        evt.raw.stopPropagation();
        evt.raw.preventDefault();
    }
};

Input.prototype.handleKeyDown = function(raw) {
    if (raw.repeat && this.lastKey != null && raw.keyCode == this.last_key.raw.keyCode) {
        this._cleanup(this.last_key, true);
        return;
    }

    var evt = new KeyEvent(raw);
    var ret = false;
    for (var i = this.handlers.length - 1; i >= 0; --i) {
        var h = this.handlers[i];
        if (h.handleKeyDown != null) {
            ret = h.handleKeyDown(evt);
            if (!evt._forward) {
                break;
            } else {
                evt._reset();
            }
        }
    }
    this._cleanup(evt, ret || alwaysStopKey(raw));
    this.last_key = evt;
};

Input.prototype.handleKeyUp = function(raw) {
    var evt = new KeyEvent(raw);
    var ret = false;
    for (var i = this.handlers.length - 1; i >= 0; --i) {
        var h = this.handlers[i];
        if (h.handleKeyUp != null) {
            ret = h.handleKeyUp(evt);
            if (!evt._forward) {
                break;
            } else {
                evt._reset();
            }
        }
    }
    this._cleanup(evt, ret || alwaysStopKey(raw));
};

Input.prototype.handleMouseOver = function(raw) {
    var evt = new MouseEvent(raw);
    var ret = false;
    for (var i = this.handlers.length - 1; i >= 0; --i) {
        var h = this.handlers[i];
        if (h.handleMouseOver != null) {
            ret = h.handleMouseOver(evt);
            if (!evt._forward) {
                break;
            } else {
                evt._reset();
            }
        }
    }
    this._cleanup(evt, ret || alwaysStopMouse(raw));
};

Input.prototype.handleMouseOut = function(raw) {
    var evt = new MouseEvent(raw);
    var ret = false;
    for (var i = this.handlers.length - 1; i >= 0; --i) {
        var h = this.handlers[i];
        if (h.handleMouseOut != null) {
            ret = h.handleMouseOut(evt);
            if (!evt._forward) {
                break;
            } else {
                evt._reset();
            }
        }
    }
    this._cleanup(evt, ret || alwaysStopMouse(raw));
};

Input.prototype.handleMouseMove = function(raw) {
    var evt = new MouseEvent(raw);
    var ret = false;
    for (var i = this.handlers.length - 1; i >= 0; --i) {
        var h = this.handlers[i];
        if (h.handleMouseMove != null) {
            ret = h.handleMouseMove(evt);
            if (!evt._forward) {
                break;
            } else {
                evt._reset();
            }
        }
    }
    this._cleanup(evt, ret || alwaysStopMouse(raw));
};

Input.prototype.handleMouseDown = function(raw) {
    var evt = new MouseEvent(raw);
    var ret = false;
    for (var i = this.handlers.length - 1; i >= 0; --i) {
        var h = this.handlers[i];
        if (h.handleMouseDown != null) {
            ret = h.handleMouseDown(evt);
            if (!evt._forward) {
                break;
            } else {
                evt._reset();
            }
        }
    }
    this._cleanup(evt, ret || alwaysStopMouse(raw));
};

Input.prototype.handleMouseUp = function(raw) {
    var evt = new MouseEvent(raw);
    var ret = false;
    for (var i = this.handlers.length - 1; i >= 0; --i) {
        var h = this.handlers[i];
        if (h.handleMouseUp != null) {
            ret = h.handleMouseUp(evt);
            if (!evt._forward) {
                break;
            } else {
                evt._reset();
            }
        }
    }
    this._cleanup(evt, ret || alwaysStopMouse(raw));
};

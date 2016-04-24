var Config = require('config').Config;



function asmEncodeKey(code, shift) {
    var adj = shift ? 10 : 0;
    switch (Config.keybindings.get()[code]) {
        case 'move_left':   return 0 + adj;
        case 'move_right':  return 1 + adj;
        case 'move_up':     return 2 + adj;
        case 'move_down':   return 3 + adj;

        // TODO: enter key handling is a hack
        case 'chat':        return 20;
        case 'cancel':      return 21;

        case 'hotbar_1':    return 31;
        case 'hotbar_2':    return 32;
        case 'hotbar_3':    return 33;
        case 'hotbar_4':    return 34;
        case 'hotbar_5':    return 35;
        case 'hotbar_6':    return 36;
        case 'hotbar_7':    return 37;
        case 'hotbar_8':    return 38;
        case 'hotbar_9':    return 39;

        case 'debug_show_panel': return 114;

        default:            return null;
    }
}

function asmDispatchKey(asm_client, code, shift) {
    var asm_code = asmEncodeKey(code, shift);
    if (asm_code == null) {
        return false;
    }
    return asm_client.inputKey(asm_code);
}


/** @constructor */
function Keyboard(asm_client) {
    // Include a no-op handler, so we can always assume the stack is nonempty.
    this._handler_stack = [function() { return false; }];
    this.monitor = null;

    var debounce = Config.debounce_time.get();
    if (debounce > 0) {
        this._debounce_timers = {};
    } else {
        this._debounce_timers = null;
    }

    var this_ = this;
    var asm_handled = {};

    this._keydown_listener = function(evt) {
        if (this_.monitor != null) {
            this_.monitor(true, evt);
        }

        if (this_._debounce_timers != null) {
            var timers = this_._debounce_timers;
            if (timers[evt.keyCode] != null) {
                clearTimeout(timers[evt.keyCode]);
                delete timers[evt.keyCode];
                evt.preventDefault();
                evt.stopPropagation();
                return;
            }
        }

        if (asmDispatchKey(asm_client, evt.keyCode, evt.shiftKey)) {
            asm_handled[evt.keyCode] = true;
            evt.preventDefault();
            evt.stopPropagation();
            return;
        }

        if (this_._topHandler()(true, evt)) {
            evt.preventDefault();
            evt.stopPropagation();
        }
    };

    this._keyup_listener = function(evt) {
        if (this_.monitor != null) {
            this_.monitor(false, evt);
        }

        if (this_._debounce_timers != null) {
            var timers = this_._debounce_timers;
            timers[evt.keyCode] = setTimeout(function() {
                delete timers[evt.keyCode];
                this_._topHandler()(false, evt);
            }, Config.debounce_time.get());
            evt.preventDefault();
            evt.stopPropagation();
            return;
        }

        if (asm_handled[evt.keyCode]) {
            delete asm_handled[evt.keyCode];
            evt.preventDefault();
            evt.stopPropagation();
            return;
        }

        if (this_._topHandler()(false, evt)) {
            evt.preventDefault();
            evt.stopPropagation();
        }
    };
}
exports.Keyboard = Keyboard;

Keyboard.prototype.pushHandler = function(handler) {
    this._handler_stack.push(handler);
}

Keyboard.prototype.popHandler = function() {
    this._handler_stack.pop();
    console.assert(this._handler_stack.length > 0);
}

Keyboard.prototype._topHandler = function() {
    var idx = this._handler_stack.length - 1;
    return this._handler_stack[idx];
}

Keyboard.prototype.attach = function(elt) {
    elt.addEventListener('keydown', this._keydown_listener);
    elt.addEventListener('keyup', this._keyup_listener);
}

Keyboard.prototype.detach = function(elt) {
    elt.removeEventListener('keydown', this._keydown_listener);
    elt.removeEventListener('keyup', this._keyup_listener);
}

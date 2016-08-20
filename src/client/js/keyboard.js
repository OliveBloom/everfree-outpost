var Config = require('config').Config;



function asmEncodeKey(code) {
    switch (Config.keybindings.get()[code]) {
        case 'move_left':   return 0;
        case 'move_right':  return 1;
        case 'move_up':     return 2;
        case 'move_down':   return 3;
        case 'run':         return 4;

        case 'interact':    return 10;
        case 'use_item':    return 11;
        case 'use_ability': return 12;
        case 'inventory':   return 13;
        case 'abilities':   return 14;

        case 'debug_show_panel':    return 20;
        case 'toggle_cursor':       return 21;

        // TODO: enter key handling is a hack
        case 'chat':        return 30;
        case 'cancel':      return 31;

        case 'hotbar_1':    return 41;
        case 'hotbar_2':    return 42;
        case 'hotbar_3':    return 43;
        case 'hotbar_4':    return 44;
        case 'hotbar_5':    return 45;
        case 'hotbar_6':    return 46;
        case 'hotbar_7':    return 47;
        case 'hotbar_8':    return 48;
        case 'hotbar_9':    return 49;

        case 'debug_log_switch':    return 99;

        default:            return null;
    }
}

function asmDispatchKey(asm_client, dir, code, shift) {
    var asm_code = asmEncodeKey(code, shift);
    if (asm_code == null) {
        return false;
    }
    if (dir) {
        return asm_client.inputKeyDown(asm_code, shift);
    } else {
        return asm_client.inputKeyUp(asm_code, shift);
    }
}

function asmKeyHandler(asm_client, down, evt) {
    var active_tag = document.activeElement.tagName.toLowerCase();
    var typing = active_tag == 'input' || active_tag == 'textarea';
    if (!typing && asmDispatchKey(asm_client, down, evt.keyCode, evt.shiftKey)) {
        return true;
    }
}
exports.asmKeyHandler = asmKeyHandler;


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

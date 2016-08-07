var Config = require('config').Config;
var decodeUtf8 = require('util/misc').decodeUtf8;

// DEPRECATED                   0x0001;
// DEPRECATED                   0x0002;
var OP_PING =                   0x0003;
var OP_INPUT =                  0x0004;
// DEPRECATED                   0x0005;
// DEPRECATED                   0x0006;
// DEPRECATED                   0x0007;
// DEPRECATED                   0x0008;
var OP_CRAFT_RECIPE =           0x0009;
var OP_CHAT =                   0x000a;
// DEPRECATED                   0x000b;
var OP_INTERACT =               0x000c;
var OP_USE_ITEM =               0x000d;
var OP_USE_ABILITY =            0x000e;
// DEPRECATED                   0x000f;
var OP_INTERACT_WITH_ARGS =     0x0010;
var OP_USE_ITEM_WITH_ARGS =     0x0011;
var OP_USE_ABILITY_WITH_ARGS =  0x0012;
var OP_MOVE_ITEM =              0x0013;
// AUTH ONLY                    0x0014;
var OP_CREATE_CHARACTER =       0x0015;
var OP_READY =                  0x0016;
var OP_CLOSE_DIALOG =           0x0017;

var OP_TERRAIN_CHUNK =          0x8001;
// DEPRECATED                   0x8002;
var OP_PONG =                   0x8003;
// DEPRECATED                   0x8004;
var OP_INIT =                   0x8005;
var OP_KICK_REASON =            0x8006;
var OP_UNLOAD_CHUNK =           0x8007;
var OP_OPEN_DIALOG =            0x8008;
// DEPRECATED                   0x8009;
var OP_OPEN_CRAFTING =          0x800a;
var OP_CHAT_UPDATE =            0x800b;
var OP_ENTITY_APPEAR =          0x800c;
var OP_ENTITY_GONE =            0x800d;
var OP_REGISTER_RESULT =        0x800e;
var OP_STRUCTURE_APPEAR =       0x800f;
var OP_STRUCTURE_GONE =         0x8010;
var OP_MAIN_INVENTORY =         0x8011;
var OP_ABILITY_INVENTORY =      0x8012;
var OP_PLANE_FLAGS =            0x8013;
var OP_GET_INTERACT_ARGS =      0x8014;
var OP_GET_USE_ITEM_ARGS =      0x8015;
var OP_GET_USE_ABILITY_ARGS =   0x8016;
var OP_SYNC_STATUS =            0x8017;
var OP_STRUCTURE_REPLACE =      0x8018;
var OP_INVENTORY_UPDATE =       0x8019;
var OP_INVENTORY_APPEAR =       0x801a;
var OP_INVENTORY_GONE =         0x801b;
var OP_ENTITY_MOTION_START =    0x801c;
var OP_ENTITY_MOTION_END =      0x801d;
var OP_ENTITY_MOTION_START_END =0x801e;
var OP_PROCESSED_INPUTS =       0x801f;
var OP_ACTIVITY_CHANGE =        0x8020;
// AUTH ONLY                    0x8021;
// AUTH ONLY                    0x8022;
var OP_INIT_NO_PAWN =           0x8023;
var OP_OPEN_PONYEDIT =          0x8024;
var OP_ENTITY_ACTIVITY_ICON =   0x8025;
var OP_CANCEL_DIALOG =          0x8026;
var OP_ENERGY_UPDATE =          0x8027;

exports.SYNC_LOADING = 0;
exports.SYNC_OK = 1;
exports.SYNC_RESET = 2;
exports.SYNC_REFRESH = 3;

/** @constructor */
function Connection(x) {
    var this_ = this;

    var socket;
    if (typeof x === 'string') {
        var url = x;
        socket = new WebSocket(url);
    } else {
        socket = x;
    }
    socket.binaryType = 'arraybuffer';
    socket.onopen = function(evt) { this_._handleOpen(evt); };
    if (Config.debug_fake_lag.get() == 0) {
        socket.onmessage = function(evt) { this_._handleMessage(evt); };
    } else {
        var queue = [];
        var lag = Config.debug_fake_lag.get();
        var dev = Config.debug_fake_lag_dev.get();
        var dispatch = function() {
            this_._handleMessage(queue.shift());
        };
        socket.onmessage = function(evt) {
            queue.push(evt);
            var delay = lag + (Math.random() * 2 - 1) * dev;
            window.setTimeout(dispatch, delay);
        };
    }
    socket.onclose = function(evt) { this_._handleClose(evt); };
    if (socket.readyState == WebSocket.OPEN) {
        setTimeout(function() { this_._handleOpen(null); }, 0);
    }
    this.socket = socket;

    this._last_kick_reason = null;

    this.onOpen = null;
    this.onClose = null;
    this.onPong = null;
    this.onChatUpdate = null;
    this.onGetInteractArgs = null;
    this.onGetUseItemArgs = null;
    this.onGetUseAbilityArgs = null;
    this.onSyncStatus = null;
    this.onOpenPonyEdit = null;

    this._asm = null;
}
exports.Connection = Connection;

Connection.prototype._send = function(msg) {
    this.socket.send(msg);
}

Connection.prototype._handleOpen = function(evt) {
    if (this.onOpen != null) {
        this.onOpen(evt);
    }
};

Connection.prototype._handleClose = function(evt) {
    if (this.onClose != null) {
        this.onClose(evt, this._last_kick_reason);
    }
};

Connection.prototype._handleMessage = function(evt) {
    var view = new DataView(evt.data);
    var offset = 0;

    function get8() {
        var result = view.getUint8(offset);
        offset += 1;
        return result;
    }

    function get16() {
        var result = view.getUint16(offset, true);
        offset += 2;
        return result;
    }

    function getI16() {
        var result = view.getInt16(offset, true);
        offset += 2;
        return result;
    }

    function get32() {
        var result = view.getUint32(offset, true);
        offset += 4;
        return result;
    }

    function getString() {
        var len = get16();
        var result = decodeUtf8(new Uint8Array(view.buffer, offset, len));
        offset += len;
        return result;
    }

    function getArg() {
        var tag = get8();
        switch (tag) {
            case 0: return get32();
            case 1: return getString();

            case 2:
                var len = get16();
                var arr = new Array(len);
                for (var i = 0; i < len; ++i) {
                    arr[i] = getArg();
                }
                return arr;

            case 3:
                var len = get16();
                var map = new Object();
                for (var i = 0; i < len; ++i) {
                    var k = getArg();
                    var v = getArg();
                    map[k] = v;
                }
                return map;
        }
    }

    var opcode = get16();

    switch (opcode) {
        case OP_PONG:
            if (this.onPong != null) {
                var msg = get16();
                var server_time = get16();
                this.onPong(msg, server_time, evt.timeStamp);
            }
            break;

        case OP_KICK_REASON:
            var msg = getString();
            this._last_kick_reason = msg;
            break;

        case OP_CHAT_UPDATE:
            if (this.onChatUpdate != null) {
                var msg = getString();
                this.onChatUpdate(msg);
            }
            break;

        case OP_GET_INTERACT_ARGS:
            if (this.onGetInteractArgs != null) {
                var dialog_id = get32();
                var args = getArg();
                this.onGetInteractArgs(dialog_id, args);
            }
            break;

        case OP_GET_USE_ITEM_ARGS:
            if (this.onGetUseItemArgs != null) {
                var item_id = get16();
                var dialog_id = get32();
                var args = getArg();
                this.onGetUseItemArgs(item_id, dialog_id, args);
            }
            break;

        case OP_GET_USE_ABILITY_ARGS:
            if (this.onGetUseItemArgs != null) {
                var item_id = get16();
                var dialog_id = get32();
                var args = getArg();
                this.onGetUseAbilityArgs(item_id, dialog_id, args);
            }
            break;

        case OP_SYNC_STATUS:
            if (this.onSyncStatus != null) {
                var synced = get8();
                this.onSyncStatus(synced);
            }
            break;

        case OP_OPEN_PONYEDIT:
            if (this.onOpenPonyEdit != null) {
                var name = getString();
                console.log('ponyedit', name);
                this.onOpenPonyEdit(name);
            };
            break;

        default:
            this._asm.handleMessage(new Uint8Array(evt.data));
            return;
    }

    console.assert(offset == view.buffer.byteLength,
            'received message with bad length (opcode ' + opcode.toString(16) + ')');
};


/** @constructor */
function MessageBuilder(length) {
    this._buf = new DataView(new ArrayBuffer(length));
    this._offset = 0;
}

MessageBuilder.prototype.put8 = function(n) {
    this._buf.setUint8(this._offset, n);
    this._offset += 1;
};

MessageBuilder.prototype.put16 = function(n) {
    this._buf.setUint16(this._offset, n, true);
    this._offset += 2;
};

MessageBuilder.prototype.put32 = function(n) {
    this._buf.setUint32(this._offset, n, true);
    this._offset += 4;
};

MessageBuilder.prototype.putString = function(s) {
    var utf8 = unescape(encodeURIComponent(s));
    this.put16(utf8.length);
    for (var i = 0; i < utf8.length; ++i) {
        this.put8(utf8.charCodeAt(i));
    }
};

MessageBuilder.prototype.putArg = function(a) {
    switch (typeof(a)) {
        case 'boolean':
        case 'number':
            this.put8(0);
            this.put32(a);
            break;

        case 'string':
            this.put8(1);
            this.putString(a);
            break;

        default:
            if (a.constructor == Array) {
                this.put8(2);
                this.put16(a.length);
                for (var i = 0; i < a.length; ++i) {
                    this.putArg(a[i]);
                }
            } else {
                this.put8(3);
                var props = Object.getOwnPropertyNames(a);
                this.put16(props.length);
                for (var i = 0; i < props.length; ++i) {
                    this.putArg(props[i]);
                    this.putArg(a[props[i]]);
                }
            }
            break;
    }
}

MessageBuilder.prototype.done = function() {
    var buf = new Uint8Array(this._buf.buffer, 0, this._offset);
    return buf;
};

MessageBuilder.prototype.reset = function() {
    this._offset = 0;
    return this;
};


var MESSAGE_BUILDER = new MessageBuilder(8192);


Connection.prototype.sendPing = function(data) {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_PING);
    msg.put16(data);
    this._send(msg.done());
};

Connection.prototype.sendInput = function(time, input) {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_INPUT);
    msg.put16(time);
    msg.put16(input);
    this._send(msg.done());
};

Connection.prototype.sendCraftRecipe = function(station_id, inventory_id, recipe_id, count) {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_CRAFT_RECIPE);
    msg.put32(station_id);
    msg.put32(inventory_id);
    msg.put16(recipe_id);
    msg.put16(count);
    this._send(msg.done());
};

Connection.prototype.sendChat = function(text) {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_CHAT);
    msg.putString(text);
    this._send(msg.done());
};

Connection.prototype.sendInteract = function(time) {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_INTERACT);
    msg.put16(time);
    this._send(msg.done());
};

Connection.prototype.sendUseItem = function(time, item_id) {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_USE_ITEM);
    msg.put16(time);
    msg.put16(item_id);
    this._send(msg.done());
};

Connection.prototype.sendUseAbility = function(time, item_id) {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_USE_ABILITY);
    msg.put16(time);
    msg.put16(item_id);
    this._send(msg.done());
};

Connection.prototype.sendInteractWithArgs = function(time, args) {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_INTERACT_WITH_ARGS);
    msg.put16(time);
    msg.putArg(args);
    this._send(msg.done());
};

Connection.prototype.sendUseItemWithArgs = function(time, item_id, args) {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_USE_ITEM_WITH_ARGS);
    msg.put16(time);
    msg.put16(item_id);
    msg.putArg(args);
    this._send(msg.done());
};

Connection.prototype.sendUseAbilityWithArgs = function(time, item_id, args) {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_USE_ABILITY_WITH_ARGS);
    msg.put16(time);
    msg.put16(item_id);
    msg.putArg(args);
    this._send(msg.done());
};

Connection.prototype.sendMoveItem = function(
        from_inventory, from_slot, to_inventory, to_slot, amount) {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_MOVE_ITEM);
    msg.put32(from_inventory);
    msg.put8(from_slot);
    msg.put32(to_inventory);
    msg.put8(to_slot);
    msg.put8(amount);
    this._send(msg.done());
};

Connection.prototype.sendCreateCharacter = function(appearance) {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_CREATE_CHARACTER);
    msg.put32(appearance);
    this._send(msg.done());
};

Connection.prototype.sendReady = function() {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_READY);
    this._send(msg.done());
};

Connection.prototype.sendCloseDialog = function() {
    var msg = MESSAGE_BUILDER.reset();
    msg.put16(OP_CLOSE_DIALOG);
    this._send(msg.done());
};

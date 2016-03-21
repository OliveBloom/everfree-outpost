var Config = require('config').Config;
var ItemDef = require('data/items').ItemDef;
var FontMetrics = require('data/fontmetrics').FontMetrics;
var W = require('ui_gl/widget');
var ItemDisplay = require('ui_gl/inventory').ItemDisplay;


var HOTBAR_SLOT_COLORS = ['yellow', 'green', 'blue', 'red'];

/** @constructor */
function HotbarSlot() {
    W.Widget.call(this);

    this.slot = new ItemDisplay();
    this.color = 0;

    this.addChild(this.slot);
}
HotbarSlot.prototype = Object.create(W.Widget.prototype);
HotbarSlot.prototype.constructor = HotbarSlot;

HotbarSlot.prototype.setItem = function(item_id) {
    this.slot.setItem(item_id);
};

HotbarSlot.prototype.setQuantity = function(qty) {
    this.slot.setQuantity(qty);
};

HotbarSlot.prototype.setColor = function(color) {
    if (this.color != color) {
        this.color = color;
        this.damage();
    }
};

HotbarSlot.prototype.runLayout = function() {
    this.slot.runLayout();
    this.slot._x = 4;
    this.slot._y = 4;
    this._width = this.slot._width + 2 * 4;
    this._height = this.slot._height + 2 * 4;
};

HotbarSlot.prototype.render = function(buf, x, y) {
    buf.drawUI('hotbar-box-' + HOTBAR_SLOT_COLORS[this.color], x, y);
};


/** @constructor */
function Hotbar() {
    W.Widget.call(this);
    this.layout = new W.ColumnLayout(1);

    var this_ = this;

    this.item_ids = new Array(9);
    this.is_item = new Array(9);
    this.boxes = new Array(9);
    for (var i = 0; i < 9; ++i) {
        this.item_ids[i] = -1;
        // Suppress quantity display for unused slots.
        this.is_item[i] = false;

        var box = new HotbarSlot();
        this.boxes[i] = box;

        (function(i) {
            box.addListener('dropcheck', function(type) {
                // TODO: support abilities as well
                return type == 'inv_items';
            });

            box.addListener('drop', function(type, data, input) {
                console.assert(type == 'inv_items');
                this_.setSlot(i, data.item_id, true);
                this_.selectSlot(i);

                if (input.drag_source.resetSlot) {
                    input.drag_source.resetSlot(data.slot);
                }
            });
        })(i);
    }

    this.active_item = -1;
    this.active_ability = -1;
    
    this.item_inv = null;
    this.ability_inv = null;

    this.addChild(new W.Spacer(0, 7));
    for (var i = 0; i < 9; ++i) {
        this.addChild(this.boxes[i]);
    }
    this.addChild(new W.Spacer(0, 7));
}
Hotbar.prototype = Object.create(W.Widget.prototype);
Hotbar.prototype.constructor = Hotbar;
exports.Hotbar = Hotbar;

Hotbar.prototype._setSlotInfo = function(idx, item_id, is_item) {
    if (is_item && this.active_ability == idx) {
        this._setActiveAbility(-1);
    }
    if (!is_item && this.active_item == idx) {
        this._setActiveItem(-1);
    }

    this.item_ids[idx] = item_id;
    this.is_item[idx] = is_item;

    var box = this.boxes[idx];
    box.setItem(item_id);
    if (is_item) {
        var qty = this.item_inv != null ? this.item_inv.count(item_id) : 0;
        box.setQuantity(qty);
    } else {
        box.setQuantity(-1);
    }
};

Hotbar.prototype.init = function() {
    var cfg = Config.hotbar.get();
    var names = cfg['names'] || [];
    var is_item_arr = cfg['is_item'] || [];

    for (var i = 0; i < names.length && i < this.item_ids.length; ++i) {
        var item = ItemDef.by_name[names[i]];
        if (item == null) {
            continue;
        }

        this._setSlotInfo(i, item.id, is_item_arr[i]);
    }

    if (cfg['active_item'] != null) {
        this._setActiveItem(cfg['active_item']);
    }
    if (cfg['active_ability'] != null) {
        this._setActiveAbility(cfg['active_ability']);
    }
};

Hotbar.prototype.setSlot = function(idx, item_id, is_item) {
    if (idx < 0 || idx >= this.item_ids.length) {
        return;
    }

    var cfg = Config.hotbar.get();
    cfg['names'][idx] = ItemDef.by_id[item_id].name;
    cfg['is_item'][idx] = is_item;
    Config.hotbar.save();

    this._setSlotInfo(idx, item_id, is_item);
};

Hotbar.prototype.selectSlot = function(idx) {
    if (idx < 0 || idx >= this.item_ids.length) {
        return;
    }
    if (this.item_ids[idx] == -1) {
        return;
    }

    if (this.is_item[idx]) {
        this._setActiveItem(idx);
    } else {
        this._setActiveAbility(idx);
    }
};

Hotbar.prototype._setActiveAbility = function(idx) {
    // Valid indices are -1 .. len-1.  -1 indicates "no selection".
    if (idx < -1 || idx >= this.item_ids.length || this.is_item[idx]) {
        return;
    }

    if (this.active_ability != -1) {
        this.boxes[this.active_ability].setColor(0);
    }
    this.active_ability = idx;
    if (this.active_ability != -1) {
        this.boxes[this.active_ability].setColor(2);
    }

    Config.hotbar.get()['active_ability'] = idx;
    Config.hotbar.save();
};

Hotbar.prototype._setActiveItem = function(idx) {
    // Valid indices are -1 .. len-1.  -1 indicates "no selection".
    if (idx < -1 || idx >= this.item_ids.length || !this.is_item[idx]) {
        return;
    }

    if (this.active_item != -1) {
        this.boxes[this.active_item].setColor(0);
    }
    this.active_item = idx;
    if (this.active_item != -1) {
        this.boxes[this.active_item].setColor(1);
    }

    Config.hotbar.get()['active_item'] = idx;
    Config.hotbar.save();
};

Hotbar.prototype.getAbility = function() {
    if (this.active_ability != -1) {
        return this.item_ids[this.active_ability];
    } else {
        return -1;
    }
};

Hotbar.prototype.getItem = function() {
    if (this.active_item != -1) {
        return this.item_ids[this.active_item];
    } else {
        return -1;
    }
};

Hotbar.prototype.attachAbilities = function(inv) {
    if (this.ability_inv != null) {
        this.ability_inv.release();
    }
    this.ability_inv = inv;
    // TODO: gray out abilities when they become unusable.
};

Hotbar.prototype._updateItems = function() {
    for (var i = 0; i < this.item_ids.length; ++i) {
        if (!this.is_item[i]) {
            continue;
        }

        this.boxes[i].setQuantity(this.item_inv.count(this.item_ids[i]));
    }
};

Hotbar.prototype.attachItems = function(inv) {
    if (this.item_inv != null) {
        this.item_inv.release();
    }
    this.item_inv = inv;

    this._updateItems();

    var this_ = this;
    inv.onUpdate(function(idx, old_item, new_item) {
        this_._updateItems();
    });
    // TODO: gray out items when quantity is zero.
};

Hotbar.prototype.render = function(buf, x, y) {
    buf.drawUI('hotbar-cap-top',
            x + ((this._width - 14) / 2)|0, y);
    buf.drawUI('hotbar-cap-bottom',
            x + ((this._width - 14) / 2)|0, y + this._height - 7);
    buf.drawUI('hotbar-bar',
            x + ((this._width - 10) / 2)|0,
            y + 7,
            10,
            this._height - 7 * 2);

};

var ItemDef = require('data/items').ItemDef;
var W = require('ui_gl/widget');


var ITEM_DISPLAY_SIZE = 16;

/** @constructor */
function ItemDisplay() {
    W.Widget.call(this);
    this.layout = new FixedSizeLayout(ITEM_DISPLAY_SIZE, ITEM_DISPLAY_SIZE);

    this.item_id = -1;
    this.qty = -1;
}
ItemDisplay.prototype = Object.create(W.Widget.prototype);
ItemDisplay.prototype.constructor = ItemDisplay;
exports.ItemDisplay = ItemDisplay;

ItemDisplay.prototype.setItem = function(item_id) {
    if (this.item_id != item_id) {
        this.item_id = item_id;
        this.damage();
    }
};

ItemDisplay.prototype.getItem = function() {
    return this.item_id;
};

ItemDisplay.prototype.setQuantity = function(qty) {
    if (this.qty != qty) {
        this.qty = qty;
        this.damage();
    }
};

ItemDisplay.prototype.getQuantity = function() {
    return this.qty;
};

function makeQuantityString(x) {
    if (x < 1000) {
        return '' + x;
    } else if (x < 10000) {
        var h = (x / 100)|0;
        var whole = (h / 10)|0;
        var frac = (h % 10);
        return whole + '.' + frac + 'k';
    } else {
        var k = (x / 1000)|0;
        return k + 'k';
    }
}

ItemDisplay.prototype.render = function(buf, x, y) {
    if (this.item_id != -1) {
        buf.drawItem(this.item_id, x, y);
    }

    // Don't draw qty if there is no item (id == 0 or -1)
    if (this.qty != -1 && this.item_id > 0) {
        var s = makeQuantityString(this.qty);
        var fm = FontMetrics.by_name['hotbar'];
        var w = fm.measureWidth(s);
        var qx = x + ITEM_DISPLAY_SIZE - w + 1;
        var qy = y + ITEM_DISPLAY_SIZE - fm.height + 1;
        fm.drawString(s, function(sx, sy, w, h, dx, dy) {
            buf.drawChar(sx, sy, w, h, qx + dx, qy + dy);
        });
    }
};


var ITEM_SLOT_ACTIVITY_LEVELS = ['inactive', 'semiactive', 'active'];

/** @constructor */
function ItemSlotGL() {
    W.Widget.call(this);
    this.layout = new PaddedPaneLayout(2, 2, 2, 2);

    this.item = new ItemDisplay();
    this.active = 0;
    this.addChild(this.item);
}
ItemSlotGL.prototype = Object.create(W.Widget.prototype);
ItemSlotGL.prototype.constructor = ItemSlotGL;
exports.ItemSlotGL = ItemSlotGL;

ItemSlotGL.prototype.setItem = function(item_id) {
    this.item.setItem(item_id);
}

ItemSlotGL.prototype.getItem = function() {
    return this.item.getItem();
};

ItemSlotGL.prototype.setQuantity = function(qty) {
    this.item.setQuantity(qty);
}

ItemSlotGL.prototype.getQuantity = function() {
    return this.item.getQuantity();
};

ItemSlotGL.prototype.setActive = function(active) {
    if (this.active != active) {
        this.active = active;
        this.damage();
    }
};

ItemSlotGL.prototype.render = function(buf, x, y) {
    buf.drawUI('item-slot-square-' + ITEM_SLOT_ACTIVITY_LEVELS[this.active],
            x, y);
};


/** @constructor */
function InventoryGrid(w, h, count) {
    W.Widget.call(this);

    console.assert(count <= w * h, "grid doesn't have space for all slots");

    this.grid_w = w;
    this.grid_h = h;
    this.count = count;
    this.slots = new Array(count);
    var this_ = this;
    for (var i = 0; i < count; ++i) {
        this.slots[i] = new ItemSlotGL();

        (function(i) {
            var slot = this_.slots[i];
            slot.addListener('mouseover', function() {
                this_._setSel(i);
            });

            slot.addListener('mousedown', function(evt, input) {
                if (this_.inventory_id != -1) {
                    input.startDrag(slot, evt, 'inv_items', {
                        inv_id: this_.inventory_id,
                        index: i,
                        item_id: slot.getItem(),
                        quantity: slot.getQuantity(),
                    });
                    slot.setItem(-1);
                }
            });

            slot.addListener('dropcheck', function(type) {
                return type == 'inv_items';
            });

            slot.addListener('dragcancel', function(type, data) {
                console.assert(type == 'inv_items');
                slot.setItem(data.item_id);
            });

            slot.addListener('drop', function(type, data) {
                console.assert(type == 'inv_items');
                // TODO: remove this, and instead do a real inventory move
                slot.setItem(data.item_id);
                slot.setQuantity(data.quantity);
            });
        })(i);

        this.addChild(this.slots[i]);
    }

    this.inventory_id = -1;
    this.active = false;
    this.sel_idx = 0;
    this.slots[this.sel_idx].setActive(1);
}
InventoryGrid.prototype = Object.create(W.Widget.prototype);
InventoryGrid.prototype.constructor = InventoryGrid;
exports.InventoryGrid = InventoryGrid;

InventoryGrid.prototype.setActive = function(active) {
    if (this.active != active) {
        this.active = active;
        this.slots[this.sel_idx].setActive(active ? 2 : 1);
        this.damage();
    }
};

InventoryGrid.prototype.attach = function(inv) {
    console.assert(this.count == inv.size(), "size mismatch");
    for (var i = 0; i < this.count; ++i) {
        var s = inv.getSlot(i);
        this.slots[i].setItem(s.item_id);
        this.slots[i].setQuantity(s.count);
    }

    var this_ = this;
    inv.onUpdate(function(idx, old_item, new_item) {
        this_.slots[idx].setItem(new_item.item_id);
        this_.slots[idx].setQuantity(new_item.count);
    });

    this.inventory_id = inv.getId();
};

InventoryGrid.prototype.selectedItem = function() {
    return this.slots[this.sel_idx].item.item_id;
};

InventoryGrid.prototype._setSel = function(new_idx) {
    if (this.sel_idx != new_idx) {
        this.slots[this.sel_idx].setActive(0);
        this.sel_idx = new_idx;
        this.slots[this.sel_idx].setActive(this.active ? 2 : 1);
        // Individual slots have been damaged
        return true;
    } else {
        return false;
    }
};

InventoryGrid.prototype._moveSel = function(dx, dy, mag) {
    var x = this.sel_idx % this.grid_w;
    var y = (this.sel_idx / this.grid_w)|0;

    var new_x = Math.max(0, Math.min(this.grid_w - 1, x + dx * mag));
    var new_y = Math.max(0, Math.min(this.grid_h - 1, y + dy * mag));
    var new_idx = Math.min(this.count - 1, new_x + new_y * this.grid_w);

    return this._setSel(new_idx);
};

InventoryGrid.prototype.onKey = function(evt) {
    var mag = evt.shiftKey ? 10 : 1;
    switch (evt.uiKeyName()) {
        case 'move_left': return this._moveSel(-1, 0, mag);
        case 'move_right': return this._moveSel(1, 0, mag);
        case 'move_up': return this._moveSel(0, -1, mag);
        case 'move_down': return this._moveSel(0, 1, mag);
    }
    return false;
};

InventoryGrid.prototype.runLayout = function() {
    for (var i = 0; i < this.count; ++i) {
        var s = this.slots[i];
        s.runLayout();
        s._x = (i % this.grid_w) * s._width;
        s._y = ((i / this.grid_w)|0) * s._height;
    }

    this._width = this.grid_w * this.slots[0]._width;
    this._height = this.grid_h * this.slots[0]._height;
};


/** @constructor */
function InventoryUIGL(inv) {
    W.Widget.call(this);
    this.layout = new PaddedPaneLayout(1, 1, 1, 1);

    var w = 6;
    var h = Math.ceil(inv.size() / w);
    this.grid = new InventoryGrid(w, h, inv.size());
    this.grid.attach(inv);
    this.grid.setActive(1);
    this.addChild(this.grid);
}
InventoryUIGL.prototype = Object.create(W.Widget.prototype);
InventoryUIGL.prototype.constructor = InventoryUIGL;
exports.InventoryUIGL = InventoryUIGL;

InventoryUIGL.prototype._setHotbar = function(hotbar_index) {
    this.dispatch('set_hotbar', hotbar_index - 1, this.grid.selectedItem());
};

InventoryUIGL.prototype.onKey = function(evt) {
    if (this.grid.onKey(evt)) {
        return true;
    }

    switch (evt.uiKeyName()) {
        case 'cancel':
        case 'select':
            this.dispatch('cancel');
            return true;

        case 'set_hotbar_1': this._setHotbar(1); return true;
        case 'set_hotbar_2': this._setHotbar(2); return true;
        case 'set_hotbar_3': this._setHotbar(3); return true;
        case 'set_hotbar_4': this._setHotbar(4); return true;
        case 'set_hotbar_5': this._setHotbar(5); return true;
        case 'set_hotbar_6': this._setHotbar(6); return true;
        case 'set_hotbar_7': this._setHotbar(7); return true;
        case 'set_hotbar_8': this._setHotbar(8); return true;
        case 'set_hotbar_9': this._setHotbar(9); return true;
    }
};

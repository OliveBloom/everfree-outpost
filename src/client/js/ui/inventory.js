var ItemDef = require('data/items').ItemDef;
var widget = require('ui/widget');
var util = require('util/misc');
var TAG = require('inventory').TAG;


/** @constructor */
function ItemGrid(inv, cols) {
    this.inv = inv;
    var size = inv.size();
    this.cols = cols;
    this.rows = ((size + cols - 1) / cols)|0;

    this.slots = new Array(size);
    this.dom = util.element('div', ['item-grid', 'g-col']);

    var row_doms = new Array(this.rows);
    for (var i = 0; i < this.rows; ++i) {
        row_doms[i] = util.element('div', ['g-row'], this.dom);
    }

    var this_ = this;
    for (var i = 0; i < size; ++i) {
        var s = new ItemSlot(this, i);
        s.update(inv.getSlot(i));
        (function(s) {
            s.dom.addEventListener('mouseenter', function(evt) {
                widget.requestFocus(this_);
                this_._setIndex(s.idx);
            });
        })(s);
        this.slots[i] = s;

        var row_dom = row_doms[(i / this.cols)|0];
        row_dom.appendChild(s.dom);
        if (i % cols == cols - 1) {
            this.dom.appendChild(util.element('br'));
        }
    }

    this.x = 0;
    this.y = 0;
    this.slots[0].dom.classList.add('active');

    inv.onUpdate(function(idx, old_item, new_item) {
        this_.slots[idx].update(new_item);
    });

    this.ondragfinish = null;
}
ItemGrid.prototype = Object.create(widget.Element.prototype);
ItemGrid.prototype.constructor = ItemGrid;
exports.ItemGrid = ItemGrid;

ItemGrid.prototype.onkey = function(evt) {
    var mag = evt.shiftKey ? 10 : 1;

    var new_x = this.x;
    var new_y = this.y;
    switch (evt.uiKeyName()) {
        case 'move_up': new_y -= mag; break;
        case 'move_down': new_y += mag; break;
        case 'move_left': new_x -= mag; break;
        case 'move_right': new_x += mag; break;
        default:
            return false;
    }

    if (new_x < 0) {
        new_x = 0;
    } else if (new_x >= this.cols) {
        new_x = this.cols - 1;
    }
    if (new_y < 0) {
        new_y = 0;
    } else if (new_y >= this.rows) {
        new_y = this.rows - 1;
    }

    if (new_x == this.x && new_y == this.y) {
        // Consider the event unhandled.  This lets the player move between
        // multiple grids in a list, by moving past the edge of one to get to
        // the next.
        return false;
    }
    if (evt.down) {
        this._setPos(new_x, new_y);
    }
    return true;
};

ItemGrid.prototype._setPos = function(x, y) {
    this.selection().dom.classList.remove('active');

    this.x = x;
    this.y = y;

    this.selection().dom.classList.add('active');
};

ItemGrid.prototype._setIndex = function(idx) {
    this._setPos(idx % this.cols, (idx / this.cols)|0);
};

ItemGrid.prototype._getIndex = function() {
    var idx = this.y * this.cols + this.x;
    if (idx >= this.inv.size()) {
        idx = this.inv.size() - 1;
    }
    return idx;
};

ItemGrid.prototype.selection = function() {
    return this.slots[this._getIndex()];
};

ItemGrid.prototype.selectItem = function(item_id) {
    for (var i = 0; i < this.inv.size(); ++i) {
        var info = this.inv.getSlot(i);
        if (info.item_id == item_id) {
            this._setIndex(i);
            break;
        }
    }
};

ItemGrid.prototype.registerDragSource = function(dnd, callback) {
    for (var i = 0; i < this.slots.length; ++i) {
        dnd.registerSource(this.slots[i]);
    }
    this.ondragfinish = function(source_slot, target_slot, data) {
        callback(source_slot.owner, source_slot.idx,
                 target_slot.owner, target_slot.idx,
                 data.count);
    };
};

ItemGrid.prototype.registerDragTarget = function(dnd) {
    for (var i = 0; i < this.slots.length; ++i) {
        dnd.registerTarget(this.slots[i]);
    }
};


/** @constructor */
function ItemSlot(owner, idx, info) {
    var parts = util.templateParts('item-slot');
    parts['qty'].textContent = '';
    parts['icon'].style.backgroundPosition = '-0rem -0rem';

    widget.Element.call(this, parts['top']);

    this.qty_part = parts['qty'];
    this.icon_part = parts['icon'];

    this.owner = owner;
    this.idx = idx;

    this.tag = TAG.EMPTY;
    this.id = 0;
    this.qty = 0;

    this.dragging = false;

    if (info != null) {
        this.update(info);
    }
}
ItemSlot.prototype = Object.create(widget.Element.prototype);
ItemSlot.prototype.constructor = ItemSlot;
exports.ItemSlot = ItemSlot;

ItemSlot.prototype.update = function(info) {
    this.tag = info.tag;
    this.id = info.item_id;
    this.qty = info.count;

    var new_qty_str = '';
    if (info.tag == TAG.EMPTY || info.tag == TAG.SPECIAL) {
        // Leave qty blank
    } else if (info.tag == TAG.BULK) {
        new_qty_str = '' + this.qty;
    } else {
        console.assert(false, 'bad tag:', info.tag);
    }

    var def = ItemDef.by_id[this.id];
    this.qty_part.textContent = new_qty_str;
    this.icon_part.style.backgroundPosition = '-' + def.tile_x + 'rem -' + def.tile_y + 'rem';

    if (this.dom.classList.contains('drag-source') && !this.dragging) {
        this.dom.classList.remove('drag-source');
    }
};

ItemSlot.prototype.ondragstart = function(evt) {
    if (this.tag == TAG.EMPTY) {
        return null;
    }

    var icon = this.dom.cloneNode(true);
    this.dom.classList.add('drag-source');
    this.dragging = true;
    return {
        count: this.qty,
        icon: icon,
    };
};

ItemSlot.prototype.ondragfinish = function(target, data) {
    this.dragging = false;
    if (this.owner.ondragfinish != null) {
        this.owner.ondragfinish(this, target, data);
    }
};

ItemSlot.prototype.ondragcancel = function(data) {
    this.dragging = false;
    this.dom.classList.remove('drag-source');
};


// Still used for recipe input/output display
/** @constructor */
function ItemRow(id, qty, name, icon_x, icon_y) {
    var parts = util.templateParts('item-row');
    parts['qty'].textContent = '' + qty;
    parts['icon'].style.backgroundPosition = '-' + icon_x + 'rem -' + icon_y + 'rem';
    parts['name'].textContent = name;

    widget.Element.call(this, parts['top']);

    this.id = id;
    this.qty = qty;
    this.quantityDiv = parts['qty'];
}
ItemRow.prototype = Object.create(widget.Element.prototype);
ItemRow.prototype.constructor = ItemRow;
exports.ItemRow = ItemRow;

ItemRow.prototype.setQuantity = function(qty) {
    this.qty = qty;
    this.quantityDiv.textContent = '' + qty;
};

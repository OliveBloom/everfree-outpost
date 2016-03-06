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

ItemDisplay.prototype.setQuantity = function(qty) {
    if (this.qty != qty) {
        this.qty = qty;
        this.damage();
    }
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

    if (this.qty != -1) {
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

ItemSlotGL.prototype.setQuantity = function(qty) {
    this.item.setQuantity(qty);
}

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


var Config = require('config').Config;
var ItemDef = require('data/items').ItemDef;

/** @constructor */
function Widget() {
    this.owner = null;
    this.children = [];
    this.layout = null;
    this._x = null;
    this._y = null;
    this._width = null;
    this._height = null;
}

Widget.prototype.addChild = function(w) {
    if (w.owner != null) {
        w.owner.removeChild(w);
    }
    w.owner = this;
    this.children.push(w);
};

Widget.prototype.removeChild = function(w) {
    if (w.owner !== this) {
        return;
    }
    w.owner = null;
    var index = this.children.indexOf(w);
    console.assert(index != -1, "child widget not found in this.children");
    this.children.splice(index, 1);
};

Widget.prototype.runLayout = function() {
    for (var i = 0; i < this.children.length; ++i) {
        this.children[i].runLayout();
    }
    this.layout.runLayout(this, this.children);
};

Widget.prototype.damage = function() {
    // TODO
};

Widget.prototype.calcSize = function(w, h) {
    // TODO: not sure what a reasonable default is here
};


/** @constructor */
function FixedSizeLayout(w, h) {
    this.w = w;
    this.h = h;
}

FixedSizeLayout.prototype.runLayout = function(owner, children) {
    console.assert(children.length == 0, "FixedSizeLayout does not support children");
    owner._width = this.w;
    owner._height = this.h;
};


/** @constructor */
function ColumnLayout(spacing) {
    this.spacing = spacing;
}

ColumnLayout.prototype.runLayout = function(owner, children) {
    var width = 0;
    for (var i = 0; i < children.length; ++i) {
        width = Math.max(width, children[i]._width);
    }

    var y = 0;
    for (var i = 0; i < children.length; ++i) {
        var c = children[i];
        c._x = (width - c._width) / 2;
        c._y = y;
        y += c._height + this.spacing;
    }
    if (children.length > 0) {
        y -= this.spacing;
    }

    owner._width = width;
    owner._height = y;
};


/** @constructor */
function Spacer(w, h) {
    Widget.call(this);
    this.layout = new FixedSizeLayout(w, h);
};
Spacer.prototype = Object.create(Widget.prototype);
Spacer.prototype.constructor = Spacer;


/** @constructor */
function ItemBox() {
    Widget.call(this);
    this.layout = new FixedSizeLayout(16 + 2*4, 16 + 2*4);

    this.item_id = -1;
    this.qty = -1;
    this.color = 0;
    this.disabled = false;
}
ItemBox.prototype = Object.create(Widget.prototype);
ItemBox.prototype.constructor = ItemBox;

ItemBox.prototype.setItem = function(item_id) {
    if (this.item_id != item_id) {
        this.item_id = item_id;
        this.damage();
    }
};

ItemBox.prototype.setQuantity = function(qty) {
    if (this.qty != qty) {
        this.qty = qty;
        this.damage();
    }
};

ItemBox.prototype.setDisabled = function(disabled) {
    this.disabled = disabled;
    this.damage();
};


/** @constructor */
function Hotbar() {
    Widget.call(this);
    this.layout = new ColumnLayout(1);

    this.item_ids = new Array(9);
    this.is_item = new Array(9);
    this.boxes = new Array(9);
    for (var i = 0; i < 9; ++i) {
        this.item_ids[i] = -1;
        // Suppress quantity display for unused slots.
        this.is_item[i] = false;

        this.boxes[i] = new ItemBox();
    }

    this.active_item = -1;
    this.active_ability = -1;
    
    this.item_inv = null;
    this.ability_inv = null;

    this.addChild(new Spacer(0, 7));
    for (var i = 0; i < 9; ++i) {
        this.addChild(this.boxes[i]);
    }
    this.addChild(new Spacer(0, 7));
}
Hotbar.prototype = Object.create(Widget.prototype);
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
        this.boxes[this.active_ability].color = 2;
    }
    this.active_ability = idx;
    if (this.active_ability != -1) {
        this.boxes[this.active_ability].color = 0;
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
        this.boxes[this.active_item].color = 1;
    }
    this.active_item = idx;
    if (this.active_item != -1) {
        this.boxes[this.active_item].color = 0;
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

    var this_ = this;
    inv.onUpdate(function(idx, old_item, new_item) {
        // TODO: might be slow (O(N^2)) at startup time
        this_._updateItems();
    });
    // TODO: gray out items when quantity is zero.
};


function GameUI() {
    Widget.call(this);
    this.hotbar = new Hotbar();

    this.addChild(hotbar);
}
GameUI.prototype = Object.create(Widget.prototype);
GameUI.prototype.constructor = GameUI;
exports.GameUI = GameUI;

GameUI.prototype.calcSize = function(w, h) {
    this._width = w;
    this._height = h;
};

GameUI.prototype.runLayout = function() {
    this._x = 0;
    this._y = 0;

    this.hotbar.runLayout();
    this.hotbar._x = 1;
    this.hotbar._y = 1;
};

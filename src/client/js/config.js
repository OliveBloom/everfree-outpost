var DEFAULT_CONFIG = {
    'show_controls': true,
    'show_inventory_updates': true,
    'ignore_browser_warning': false,
    'chat_scrollback': 100,
    'chat_lines': 8,
    'chat_autohide': false,
    'scale_world': 0,
    'scale_ui': 0,

    'render_outline': true,
    'render_names': true,
    'render_simplified_slicing': false,

    'motion_prediction': true,
    'input_delay': 10,
    'debounce_time': 0,

    'hotbar': {
        // 9 slots in each array
        'names': [-1, -1, -1, -1, -1, -1, -1, -1, -1],
        'is_item': [false, false, false, false, false, false, false, false, false],
        'active_item': -1,
        'active_ability': -1,
    },

    'login_name': null,
    'login_secret': null,
    'world_version': null,
    'last_appearance': null,

    'keybindings': {
        37: 'move_left',    // ArrowLeft
        39: 'move_right',   // ArrowRight
        38: 'move_up',      // ArrowUp
        40: 'move_down',    // ArrowDown
        16: 'run',          // Shift

        65: 'interact',     // A
        83: 'use_ability',  // S
        87: 'abilities',    // W
        68: 'use_item',     // D
        69: 'inventory',    // E

        112: 'show_controls', // F1
        113: 'show_menu',   // F2
        114: 'debug_show_panel', // F3
        115: 'debug_test',  // F4
        67: 'toggle_cursor', // C

        27: 'cancel',       // Esc
        32: 'cancel',       // Space
        13: 'chat',         // Enter
        191: 'chat_command', // '/'

        49: 'hotbar_1',
        50: 'hotbar_2',
        51: 'hotbar_3',
        52: 'hotbar_4',
        53: 'hotbar_5',
        54: 'hotbar_6',
        55: 'hotbar_7',
        56: 'hotbar_8',
        57: 'hotbar_9',
    },

    'chat_keybindings': {
        13: 'send',         // Enter
        27: 'cancel',       // Esc
    },

    'ui_keybindings': {
        37: 'move_left',    // ArrowLeft
        39: 'move_right',   // ArrowRight
        38: 'move_up',      // ArrowUp
        40: 'move_down',    // ArrowDown
        27: 'cancel',       // Esc
        32: 'cancel',       // Space
        13: 'select',       // Enter
        65: 'select',       // A

        49: 'set_hotbar_1',
        50: 'set_hotbar_2',
        51: 'set_hotbar_3',
        52: 'set_hotbar_4',
        53: 'set_hotbar_5',
        54: 'set_hotbar_6',
        55: 'set_hotbar_7',
        56: 'set_hotbar_8',
        57: 'set_hotbar_9',
    },

    'show_key_display': false,

    'ignores': {},

    'debug_show_panel': false,
    'debug_force_mobile_warning': false,
    'debug_force_browser_warning': false,
    'debug_block_webgl_extensions': {},
    'debug_shader_defs': {},
    'debug_fake_lag': 0,
    'debug_fake_lag_dev': 0,
};


exports.Config = {
    show_controls: new ConfigItem('show_controls'),
    show_inventory_updates: new ConfigItem('show_inventory_updates'),
    ignore_browser_warning: new ConfigItem('ignore_browser_warning'),
    chat_scrollback: new ConfigItem('chat_scrollback'),
    chat_lines: new ConfigItem('chat_lines'),
    chat_autohide: new ConfigItem('chat_autohide'),
    scale_world: new ConfigItem('scale_world'),
    scale_ui: new ConfigItem('scale_ui'),

    render_outline: new ConfigItem('render_outline'),
    render_names: new ConfigItem('render_names'),
    render_simplified_slicing: new ConfigItem('render_simplified_slicing'),

    motion_prediction: new ConfigItem('motion_prediction'),
    input_delay: new ConfigItem('input_delay'),
    debounce_time: new ConfigItem('debounce_time'),

    hotbar: new ConfigItem('hotbar'),

    login_name: new ConfigItem('login_name'),
    login_secret: new ConfigItem('login_secret'),
    world_version: new ConfigItem('world_version'),
    last_appearance: new ConfigItem('last_appearance'),

    keybindings: new ConfigItem('keybindings'),
    chat_keybindings: new ConfigItem('chat_keybindings'),
    ui_keybindings: new ConfigItem('ui_keybindings'),

    show_key_display: new ConfigItem('show_key_display'),

    ignores: new ConfigItem('ignores'),

    debug_show_panel: new ConfigItem('debug_show_panel'),
    debug_force_mobile_warning: new ConfigItem('debug_force_mobile_warning'),
    debug_force_browser_warning: new ConfigItem('debug_force_browser_warning'),
    debug_block_webgl_extensions: new ConfigItem('debug_block_webgl_extensions'),
    debug_shader_defs: new ConfigItem('debug_shader_defs'),
    debug_fake_lag: new ConfigItem('debug_fake_lag'),
    debug_fake_lag_dev: new ConfigItem('debug_fake_lag_dev'),
};


/** @constructor */
function ConfigItem(key) {
    this.key = key;
    this.value = null;
}

ConfigItem.prototype.get = function() {
    if (this.value == null) {
        var str = localStorage.getItem(this.key);
        if (!str) {
            this.value = DEFAULT_CONFIG[this.key];
        } else {
            this.value = JSON.parse(str);
        }
    }

    return this.value;
};

ConfigItem.prototype.set = function(value) {
    this.value = value;
    this.save();
};

ConfigItem.prototype.toggle = function(value) {
    var new_value = !this.get();
    this.set(new_value);
    return new_value;
};

ConfigItem.prototype.isSet = function() {
    return localStorage.getItem(this.key) != null;
};

ConfigItem.prototype.reset = function() {
    localStorage.removeItem(this.key);
    this.value = null;
};

ConfigItem.prototype.save = function() {
    localStorage.setItem(this.key, JSON.stringify(this.value));
};


function rawGet(key) {
    var parts = key.split('.');

    var base_key = parts[0];
    var base_str = localStorage.getItem(base_key);
    var obj;
    if (!base_str) {
        obj = DEFAULT_CONFIG[base_key];
    } else {
        obj = JSON.parse(base_str);
    }

    for (var i = 1; i < parts.length; ++i) {
        obj = obj[parts[i]];
    }

    return obj;
};
exports.rawGet = rawGet;

function rawSet(key, val) {
    var parts = key.split('.');
    var base_key = parts[0];

    if (parts.length > 1) {
        var base_str = localStorage.getItem(base_key)
        var base_obj;
        if (!base_str) {
            base_obj = DEFAULT_CONFIG[base_key];
        } else {
            base_obj = JSON.parse(base_str);
        }

        var obj = base_obj;
        for (var i = 1; i < parts.length - 1; ++i) {
            obj = obj[parts[i]];
        }

        if (typeof obj[parts[parts.length - 1]] === 'number') {
            val = +val;
        }
        obj[parts[parts.length - 1]] = val;

        localStorage.setItem(base_key, JSON.stringify(base_obj));
    } else {
        var val_str;
        if (typeof DEFAULT_CONFIG[base_key] === 'number') {
            val_str = JSON.stringify(+val);
        } else {
            val_str = JSON.stringify(val);
        }
        localStorage.setItem(base_key, val_str);
    }
};
exports.rawSet = rawSet;

function rawClear(key, val) {
    var parts = key.split('.');
    var base_key = parts[0];

    if (parts.length > 1) {
        var obj = JSON.parse(localStorage.getItem(base_key));
        var def = DEFAULT_CONFIG[base_key];

        for (var i = 1; i < parts.length - 1; ++i) {
            obj = obj[parts[i]];
            def = def[parts[i]];
        }
        obj[parts[parts.length - 1]] = def[parts[parts.length - 1]];

        localStorage.setItem(base_key, JSON.stringify(obj));
    } else {
        localStorage.removeItem(base_key);
    }
};
exports.rawClear = rawClear;

var DEFAULT_CONFIG = {
    'show_controls': true,
    'ignore_browser_warning': false,

    'keybindings': {
        37: 'move_left',    // ArrowLeft
        39: 'move_right',   // ArrowRight
        38: 'move_up',      // ArrowUp
        40: 'move_down',    // ArrowDown
        16: 'run',          // Shift
        65: 'interact',     // A
        68: 'use_item',     // D
        69: 'inventory',    // E
        112: 'show_controls', // F1
        27: 'cancel',       // Esc
        32: 'cancel',       // Space
    },

    'debug_timing_delay': [0, 0],
    'debug_force_mobile_warning': false,
    'debug_force_browser_warning': false,
};


exports.Config = {
    show_controls: new ConfigItem('show_controls'),
    ignore_browser_warning: new ConfigItem('ignore_browser_warning'),
    keybindings: new ConfigItem('keybindings'),

    debug_timing_delay: new ConfigItem('debug_timing_delay'),
    debug_force_mobile_warning: new ConfigItem('debug_force_mobile_warning'),
    debug_force_browser_warning: new ConfigItem('debug_force_browser_warning'),
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
    localStorage.setItem(this.key, JSON.stringify(value));
};

ConfigItem.prototype.toggle = function(value) {
    var new_value = !this.get();
    this.set(new_value);
    return new_value;
};

var config = require('config');

var EXPORTS = {
    'Config': config.Config,
    'addConfigItem': config.addConfigItem,
    'rawGet': config.rawGet,
    'rawSet': config.rawSet,
    'rawClear': config.rawClear,

    // To be initialized later
    'asm': null,
    'assets': null,
};

exports.EXPORTS = EXPORTS;
window['OUTPOST'] = EXPORTS;

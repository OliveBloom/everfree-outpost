var util = require('util/misc');

function loadPack(blob, next) {
    new PackReader(blob, next);
}
exports.loadPack = loadPack;

/** @constructor */
function PackReader(blob, next) {
    this.blob = blob;
    this.fr = new FileReader();
    this.current = 0;
    this.index_bytes = null;
    this.index = null;
    this.assets = {};
    this.next = next;

    var this_ = this;

    this._startReadIndexLength(this.blob.slice(0, 4));
}

PackReader.prototype._startReadIndexLength = function() {
    var this_ = this;
    this.fr.onloadend = function() { this_._finishReadIndexLength(); };
    this.fr.readAsArrayBuffer(this.blob.slice(0, 4));
};

PackReader.prototype._finishReadIndexLength = function() {
    checkError(this.fr.error, 'pack index length');
    var data = new DataView(this.fr.result);
    this.index_bytes = data.getUint32(0, true);
    this._startReadIndex();

    console.log('index length: 0x', this.index_bytes.toString(16));
};

PackReader.prototype._startReadIndex = function() {
    var this_ = this;
    this.fr.onloadend = function() { this_._finishReadIndex(); };
    this.fr.readAsText(this.blob.slice(4, 4 + this.index_bytes));
};

PackReader.prototype._finishReadIndex = function() {
    checkError(this.fr.error, 'pack index');
    this.index = JSON.parse(this.fr.result);
    this._startReadItem();
};

PackReader.prototype._startReadItem = function() {
    if (this.current >= this.index.length) {
        // Done reading items.
        this.next(this.assets);
        return;
    }

    var entry = this.index[this.current];
    switch (entry['type']) {
        case 'json':
            this._startReadJson(entry);
            break;
        case 'text':
            this._startReadText(entry);
            break;
        case 'image':
            this._startReadImage(entry);
            break;
        case 'binary':
            this._startReadBinary(entry);
            break;
        case 'url':
            this._startReadUrl(entry);
            break;
        default:
            throw ('Pack entry ' + entry['name'] + ' has invalid type "' + entry['type'] + '"');
    }
};

PackReader.prototype._finishReadItem = function() {
    ++this.current;
    this._startReadItem();
};


PackReader.prototype._startReadJson = function(entry) {
    var this_ = this;
    this.fr.onloadend = function() { this_._finishReadJson(entry['name']); };
    var base = 4 + this.index_bytes + entry['offset'];
    this.fr.readAsText(this.blob.slice(base, base + entry['length']));
};

PackReader.prototype._finishReadJson = function(name) {
    checkError(this.fr.error, name);
    this.assets[name] = JSON.parse(this.fr.result);
    this._finishReadItem();
};

PackReader.prototype._startReadText = function(entry) {
    var this_ = this;
    this.fr.onloadend = function() { this_._finishReadText(entry['name']); };
    var base = 4 + this.index_bytes + entry['offset'];
    this.fr.readAsText(this.blob.slice(base, base + entry['length']));
};

PackReader.prototype._finishReadText = function(name) {
    checkError(this.fr.error, name);
    this.assets[name] = this.fr.result;
    this._finishReadItem();
};

PackReader.prototype._startReadImage = function(entry) {
    var base = 4 + this.index_bytes + entry['offset'];
    var url = window.URL.createObjectURL(this.blob.slice(base, base + entry['length']));

    var this_ = this;
    var img = util.element('img', ['src=' + url]);
    img.onload = function() { this_._finishReadImage(entry['name'], img); };
    img.onerror = function() {
        throw ('Error reading ' + entry['name'] + ': image loading failed');
    };
};

PackReader.prototype._finishReadImage = function(name, img) {
    this.assets[name] = img;
    img.onload = null;
    img.onerror = null;
    this._finishReadItem();
};

PackReader.prototype._startReadBinary = function(entry) {
    var this_ = this;
    this.fr.onloadend = function() { this_._finishReadBinary(entry['name']); };
    var base = 4 + this.index_bytes + entry['offset'];
    this.fr.readAsArrayBuffer(this.blob.slice(base, base + entry['length']));
};

PackReader.prototype._finishReadBinary = function(name) {
    checkError(this.fr.error, name);
    this.assets[name] = this.fr.result;
    this._finishReadItem();
};

PackReader.prototype._startReadUrl = function(entry) {
    var base = 4 + this.index_bytes + entry['offset'];
    var url = window.URL.createObjectURL(this.blob.slice(base, base + entry['length']));
    this.assets[entry['name']] = url;
    this._finishReadItem();
};


function checkError(err, what) {
    if (err) {
        throw ('Error reading ' + what + ': ' + err);
    }
}

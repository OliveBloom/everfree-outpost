/** @constructor */
function FontMetrics_(info) {
    this.first_char = info['first_char'];
    this.xs = info['xs'];
    this.y = info['y'];
    this.widths = info['widths'];
    this.height = info['height'];
    this.spacing = info['spacing'];
    this.space_width = info['space_width'];
}

FontMetrics_.prototype.measureWidth = function(s) {
    var total = 0;
    for (var i = 0; i < s.length; ++i) {
        var code = s.charCodeAt(i);
        var idx = code - this.first_char;

        var width;
        if (code == 0x20) {
            width = this.space_width;
        } else {
            width = this.widths[idx] || 0;
        }

        total += width;
        if (i > 0) {
            total += this.spacing;
        }
    }
    return total;
};

FontMetrics_.prototype.drawString = function(s, callback) {
    var dest_x = 0;

    for (var i = 0; i < s.length; ++i) {
        var code = s.charCodeAt(i);
        var idx = code - this.first_char;

        if (code == 0x20) {
            dest_x += this.space_width;
            continue;
        } else if (idx < 0 || idx >= this.widths.length) {
            // Invalid character
            continue;
        }

        var src_x = this.xs[idx];
        var w = this.widths[idx];
        callback(src_x, this.y, w, this.height, dest_x, 0);

        dest_x += w + this.spacing;
    }
};


var FontMetrics = {};
exports.FontMetrics = FontMetrics;

FontMetrics.by_name = {};

FontMetrics.init = function(all_info) {
    var names = Object.getOwnPropertyNames(all_info);
    for (var i = 0; i < names.length; ++i) {
        var name = names[i];
        var info = all_info[name];
        FontMetrics.by_name[name] = new FontMetrics_(info);
    }
};

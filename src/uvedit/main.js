function $(x) { return document.getElementById(x); }

function init_asm() {
    var module = window.asmlibs_code;
    var static_data = window.asmlibs_data;
    var static_size = window.asmlibs_data_size;

    var stack_start = (8 + static_size + 7) & ~7;
    var stack_end = stack_start + 16 * 1024;

    var buffer = new ArrayBuffer(1 * 1024 * 1024);

    var msg_buffer = '';    // Captured and mutated by the closures in `env`
    var env = {
        'abort': function() {
            console.assert(false, 'abort');
            throw 'abort';
        },

        'writeStr': function(ptr, len) {
            var view = new Uint8Array(buffer, ptr, len);
            var s = '';
            var saw_utf8 = false;
            for (var i = 0; i < view.length; ++i) {
                var byte_ = view[i];
                s += String.fromCharCode(byte_);
                if (byte_ >= 0x80) {
                    saw_utf8 = true;
                }
            }

            if (saw_utf8) {
                s = decodeURIComponent(escape(s));
            }

            msg_buffer += s;
        },

        'flushStr': function() {
            console.log(msg_buffer);
            msg_buffer = '';
        },

        'flushStrWarn': function() {
            console.warn(msg_buffer);
            msg_buffer = '';
        },

        'flushStrErr': function() {
            console.error(msg_buffer);
            window.onerror(msg_buffer, '<native code>', 0, 0, null);
            msg_buffer = '';
        },

        'STACK_START': stack_start,
        'STACK_END': stack_end,
    };

    var dest = new Uint8Array(buffer, 8, static_data.byteLength);
    dest.set(static_data);

    var asm = module(window, env, buffer);
    asm.buffer = buffer;
    asm.asmmalloc_init(stack_end, buffer.byteLength);
    asm.init();
    return asm
}




// Viewport

function Viewport(side, parent_) {
    this.side = side;
    this.parent_ = parent_;
    this.children = [];
    if (parent_) {
        parent_.children.push(this);
    }

    this.canvas = $('cnv-' + side);
    this.ctx = this.canvas.getContext('2d');
    this.ctx.imageSmoothingEnabled = false;
    this.ctx.mozImageSmoothingEnabled = false;
    this.scale = this.canvas.width / 96;

    this.overlay = document.createElement('canvas');
    this.overlay.width = 96;
    this.overlay.height = 96;
    this.overlay_ctx = this.overlay.getContext('2d');

    this.asm = init_asm();

    this.frame_pending = false;

    this.init_dropdown_callbacks();
}

// Utility

Viewport.prototype.get_value = function(name) {
    var val = $('sel-' + this.side + '-' + name).value;
    if (val == '*') {
        val = this.parent_.get_value(name);
    }
    return +val;
};

Viewport.prototype.get_sprite = function() {
    var layer = this.get_value('layer');
    var anim = this.get_value('anim');
    var anim_local_id = DATA.anims_c[anim].local_id;
    var layer_data = DATA.layers_c[layer];
    var gfx = layer_data.start + anim_local_id;
    return {
        gfx: DATA.graphics[gfx],
        anim: DATA.anims_c[anim],
    };
};

Viewport.prototype.current_key = function() {
    var layer = DATA.layers_s[this.get_value('layer')].name;
    var anim = DATA.anims_s[this.get_value('anim')].name;
    var frame = this.get_value('frame');
    var part = this.get_value('part');
    return {
        layer: layer,
        anim: anim,
        frame: frame,
        part: part,
    };
};

function key_to_string(k) {
    return k.layer + '$' + k.anim + '$' + k.frame + '$' + k.part;
}

// Rendering

Viewport.prototype.request_frame = function() {
    if (!this.frame_pending) {
        this.frame_pending = true;
        var this_ = this;
        window.requestAnimationFrame(function() { this_.frame(); });
    }
};

Viewport.prototype.frame = function() {
    this.frame_pending = false;
    this.asm.update();
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

    if (DATA.ready) {
        // Sprite
        var s = this.get_sprite();
        var frame = this.get_value('frame');
        var sx = s.gfx.src_offset[0] + frame * s.gfx.size[0];
        var sy = s.gfx.src_offset[1];
        var dx = s.gfx.dest_offset[0] * this.scale;
        var dy = s.gfx.dest_offset[1] * this.scale;
        var size_x = s.gfx.size[0];
        var size_y = s.gfx.size[1];

        this.ctx.drawImage(DATA.img,
                sx, sy, size_x, size_y,
                dx, dy, size_x * this.scale, size_y * this.scale);
    }

    // Overlay
    var ptr = this.asm.get_overlay_ptr();
    var len = 96 * 96;
    var view = new Uint8ClampedArray(this.asm.buffer, ptr, len * 4);

    var idat = new ImageData(view, 96, 96);
    this.overlay_ctx.putImageData(idat, 0, 0);

    this.ctx.drawImage(this.overlay,
            0, 0, 96, 96, 0, 0, this.canvas.width, this.canvas.height);

    // Lines
    var ptr = this.asm.get_lines_ptr();
    var len = this.asm.get_lines_len();
    if (len > 0) {
        var view = new Int32Array(this.asm.buffer, ptr, len * 4);

        for (var i = 0; i < view.length; i += 4) {
            var x0 = view[i + 0];
            var y0 = view[i + 1];
            var x1 = view[i + 2];
            var y1 = view[i + 3];
            this.ctx.beginPath();
            this.ctx.moveTo(x0 + 0.5, y0 + 0.5);
            this.ctx.lineTo(x1 + 0.5, y1 + 0.5);
            this.ctx.stroke();
        }
    }
};

// Save/load

Viewport.prototype.save_data = function() {
    var ptr = this.asm.get_mask_ptr();
    var view = new Uint8Array(this.asm.buffer, ptr, 96 * 96);
    var k = key_to_string(this.current_key());
    var s = '';
    for (var i = 0; i < view.length; ++i) {
        s += String.fromCharCode(view[i]);
    }
    var v = btoa(s);
    localStorage.setItem(k, v);
};

Viewport.prototype.load_data = function() {
    var ptr = this.asm.get_mask_ptr();
    var view = new Uint8Array(this.asm.buffer, ptr, 96 * 96);
    var k = key_to_string(this.current_key());
    var v = localStorage.getItem(k);
    if (v == null) {
        view.fill(1);   // set all to MASKED
    } else {
        var s = atob(v);
        for (var i = 0; i < s.length; ++i) {
            view[i] = s.charCodeAt(i);
        }
    }
};

// Dropdowns

function empty_element(elt) {
    while (elt.lastElementChild != null) {
        elt.removeChild(elt.lastElementChild);
    }
}

function mk_option(name, value) {
    var opt = document.createElement('option');
    opt.setAttribute('value', '' + value);
    opt.textContent = name;
    return opt;
}

Viewport.prototype.update_select = function(name, opts) {
    var sel = $('sel-' + this.side + '-' + name);

    var old_val = sel.value;

    empty_element(sel);
    if (this.parent_) {
        console.log(this.side, name, 'has parent', this.parent_);
        sel.appendChild(mk_option('*', '*'));
    }
    for (var i = 0; i < opts.length; ++i) {
        sel.appendChild(opts[i]);
    }

    sel.value = old_val;
    sel.value = sel.value || '*';
    sel.value = sel.value || opts[0].value;
};

Viewport.prototype.init_sprites = function(data) {
    var opts = [];
    for (var i = 0; i < data.layers_s.length; ++i) {
        var name = data.layers_s[i].name;
        if (name.startsWith('pony//') && name.indexOf('base') != -1) {
            opts.push(mk_option(name, i));
        }
    }
    this.update_select('layer', opts);

    var opts = [];
    for (var i = 0; i < data.anims_s.length; ++i) {
        var name = data.anims_s[i].name;
        if (name.startsWith('pony//')) {
            opts.push(mk_option(name, i));
        }
    }
    this.update_select('anim', opts);

    this.change_anim_layer();
};

Viewport.prototype.change_anim_layer = function() {
    var s = this.get_sprite();

    var opts = [];
    for (var i = 0; i < s.anim.length; ++i) {
        opts.push(mk_option(i + 1, i));
    }
    this.update_select('frame', opts);

    this.change_frame();
};

Viewport.prototype.change_frame = function() {
    // Load sprite data into ASM
    var s = this.get_sprite();
    var frame = this.get_value('frame');
    var sx = s.gfx.src_offset[0] + frame * s.gfx.size[0];
    var sy = s.gfx.src_offset[1];
    var dx = s.gfx.dest_offset[0];
    var dy = s.gfx.dest_offset[1];
    var size_x = s.gfx.size[0];
    var size_y = s.gfx.size[1];

    this.overlay_ctx.clearRect(0, 0, 96, 96);
    this.overlay_ctx.drawImage(DATA.img,
            sx, sy, size_x, size_y,
            dx, dy, size_x, size_y);
    var idat = this.overlay_ctx.getImageData(0, 0, 96, 96);

    var len = idat.width * idat.height;
    var ptr = this.asm.asmmalloc_alloc(len * 4, 4);
    var view = new Uint8ClampedArray(this.asm.buffer, ptr, len * 4);
    view.set(idat.data);
    this.asm.load_sprite(ptr, len);
    this.asm.asmmalloc_free(ptr);

    this.change_part();
};

Viewport.prototype.change_part = function() {
    this.load_data();
    this.request_frame();
    for (var i = 0; i < this.children.length; ++i) {
        this.children[i].change_part();
    }
};

Viewport.prototype.init_dropdown_callbacks = function() {
    var this_ = this;
    $('sel-' + this.side + '-anim').onchange =
        function() { this_.change_anim_layer(); };
    $('sel-' + this.side + '-layer').onchange =
        function() { this_.change_anim_layer(); };
    $('sel-' + this.side + '-frame').onchange =
        function() { this_.change_frame(); };
    $('sel-' + this.side + '-part').onchange =
        function() { this_.change_part(); };
};





function Data() {
    this.pending = 0;
    this.ready = false;
    this.callback = null;
}

Data.prototype._finishOne = function() {
    --this.pending;
    if (this.pending == 0) {
        this.callback(this);
        this.ready = true;
    }
};

Data.prototype.loadJson = function(key, url) {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', url, true);

    xhr.responseType = 'json';

    var this_ = this;
    xhr.onreadystatechange = function() {
        if (xhr.readyState == XMLHttpRequest.DONE) {
            this_[key] = xhr.response;
            this_._finishOne();
        }
    };

    xhr.send();
    ++this.pending;
};

Data.prototype.loadImage = function(key, url) {
    var img = document.createElement('img');
    img.setAttribute('src', url);

    var this_ = this;
    img.onload = function() {
        this_._finishOne();
    };

    this[key] = img;
    ++this.pending;
};




var MAIN = new Viewport('main');
var REF = new Viewport('ref', MAIN);

var DATA = new Data();
DATA.callback = function() {
    MAIN.init_sprites(DATA);
    REF.init_sprites(DATA);
};
DATA.loadJson('anims_s', 'animations_server.json')
DATA.loadJson('anims_c', 'animations_client.json')
DATA.loadJson('layers_s', 'sprite_layers_server.json')
DATA.loadJson('layers_c', 'sprite_layers_client.json')
DATA.loadJson('graphics', 'sprite_graphics_client.json')
DATA.loadImage('img', 'sprites0.png')




MAIN.canvas.addEventListener('mousemove', function(evt) {
    MAIN.asm.handle_mouse_move(evt.offsetX, evt.offsetY);
    MAIN.request_frame();
});

MAIN.canvas.addEventListener('mousedown', function(evt) {
    MAIN.asm.handle_mouse_down(evt.offsetX, evt.offsetY, evt.shiftKey);
    MAIN.request_frame();
});

MAIN.canvas.addEventListener('mouseup', function(evt) {
    MAIN.asm.handle_mouse_up(evt.offsetX, evt.offsetY);
    MAIN.request_frame();
    MAIN.save_data();
});

document.addEventListener('keydown', function(evt) {
    var handled = true;
    switch (evt.keyCode) {
        case 'A'.charCodeAt(0): MAIN.asm.set_mode(0); break;

        case 'O'.charCodeAt(0):
        case 'S'.charCodeAt(0): MAIN.asm.set_mode(1); break;

        case 'E'.charCodeAt(0):
        case 'D'.charCodeAt(0): MAIN.asm.set_mode(2); break;

        case 'U'.charCodeAt(0):
        case 'F'.charCodeAt(0): MAIN.asm.set_mode(3); break;

        default: handled = false; break;
    }
    if (handled) {
        evt.stopPropagation();
        evt.preventDefault();
        request_frame();
    }
});


$('btn-edit-mesh').onclick = function() { MAIN.asm.set_mode(0); };
$('btn-erase-mask').onclick = function() { MAIN.asm.set_mode(1); };
$('btn-draw-mask').onclick = function() { MAIN.asm.set_mode(2); };
$('btn-border-mask').onclick = function() { MAIN.asm.set_mode(3); };

$('btn-export').onclick = function() {
    var data = export_data();
    var blob = new Blob([data]);
    var url = URL.createObjectURL(blob);

    console.log(url);

    var a = document.createElement('a');
    a.setAttribute('href', url);
    a.setAttribute('download', 'uvedit_data.json');
    document.body.appendChild(a);
    a.textContent = 'Download';
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(data);
};

function export_data() {
    var result = {};
    for (var i = 0; i < localStorage.length; ++i) {
        var k = localStorage.key(i);
        if (k.indexOf('$') != -1) {
            result[k] = localStorage.getItem(k);
        }
    }
    return JSON.stringify(result);
}

$('btn-import').onclick = function() {
    var input = document.createElement('input');
    input.setAttribute('type', 'file');

    input.onchange = function(evt) {
        if (input.files.length != 1) {
            return;
        }
        var f = input.files[0];
        var reader = new FileReader();
        reader.onloadend = function(evt) {
            import_data(reader.result);
            location.replace(location);
        };
        reader.readAsText(f);
    };

    document.body.appendChild(input);
    input.click();
    document.body.removeChild(input);
};

function import_data(s) {
    var j = JSON.parse(s);
    localStorage.clear();
    var keys = Object.getOwnPropertyNames(j);
    for (var i = 0; i < keys.length; ++i) {
        var k = keys[i];
        localStorage.setItem(k, j[k]);
    }
}

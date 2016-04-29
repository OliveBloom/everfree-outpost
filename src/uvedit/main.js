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

var overlay = document.createElement('canvas');
overlay.width = 96;
overlay.height = 96;
var overlay_ctx = overlay.getContext('2d');

function asm_draw(ctx, asm) {
    if (DATA.ready) {
        // Sprite
        var s = get_sprite();
        var frame = +$('sel-main-frame').value;
        var sx = s.gfx.src_offset[0] + frame * s.gfx.size[0];
        var sy = s.gfx.src_offset[1];
        var dx = s.gfx.dest_offset[0] * 8;
        var dy = s.gfx.dest_offset[1] * 8;
        var size_x = s.gfx.size[0];
        var size_y = s.gfx.size[1];

        ctx.drawImage(DATA.img,
                sx, sy, size_x, size_y,
                dx, dy, size_x * 8, size_y * 8);
    }

    // Overlay
    var ptr = asm.get_overlay_ptr();
    var len = 96 * 96;
    var view = new Uint8ClampedArray(asm.buffer, ptr, len * 4);

    var idat = new ImageData(view, 96, 96);
    overlay_ctx.putImageData(idat, 0, 0);

    ctx.drawImage(overlay, 0, 0, 96, 96, 0, 0, 768, 768);

    // Lines
    var ptr = asm.get_lines_ptr();
    var len = asm.get_lines_len();
    var view = new Int32Array(asm.buffer, ptr, len * 4);

    for (var i = 0; i < view.length; i += 4) {
        var x0 = view[i + 0];
        var y0 = view[i + 1];
        var x1 = view[i + 2];
        var y1 = view[i + 3];
        ctx.beginPath();
        ctx.moveTo(x0 + 0.5, y0 + 0.5);
        ctx.lineTo(x1 + 0.5, y1 + 0.5);
        ctx.stroke();
    }
}


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


function init_sprites(data) {
    empty_element($('sel-main-layer'));
    for (var i = 0; i < data.layers_s.length; ++i) {
        var name = data.layers_s[i].name;
        if (name.startsWith('pony//') && name.indexOf('base') != -1) {
            var opt = document.createElement('option');
            opt.setAttribute('value', '' + i);
            opt.textContent = name;
            $('sel-main-layer').appendChild(opt);
        }
    }

    empty_element($('sel-main-anim'));
    for (var i = 0; i < data.anims_s.length; ++i) {
        var name = data.anims_s[i].name;
        if (name.startsWith('pony//')) {
            var opt = document.createElement('option');
            opt.setAttribute('value', '' + i);
            opt.textContent = name;
            $('sel-main-anim').appendChild(opt);

            if (name == 'pony//stand-0') {
                $('sel-main-anim').value = '' + i;
            }
        }
    }

    update_frames();
}

function get_sprite() {
    var layer = +$('sel-main-layer').value;
    var anim = +$('sel-main-anim').value;
    var anim_local_id = DATA.anims_c[anim].local_id;
    var layer_data = DATA.layers_c[layer];
    var gfx = layer_data.start + anim_local_id;
    return {
        gfx: DATA.graphics[gfx],
        anim: DATA.anims_c[anim],
    };
}

function update_frames() {
    var old_frame = +$('sel-main-frame').value;

    var s = get_sprite();

    empty_element($('sel-main-frame'));
    console.log(s, s.anim.length);
    for (var i = 0; i < s.anim.length; ++i) {
        console.log(i);
        var opt = document.createElement('option');
        opt.setAttribute('value', '' + i);
        opt.textContent = '' + (i + 1);
        $('sel-main-frame').appendChild(opt);

        if (i == old_frame) {
            $('sel-main-frame').value = '' + i;
        }
    }

    change_frame();
}

function change_frame() {
    // TODO: save/load mesh data
    request_frame();
}

$('sel-main-anim').onchange = update_frames;
$('sel-main-layer').onchange = update_frames;
$('sel-main-frame').onchange = change_frame;


var DATA = new Data();
DATA.callback = init_sprites;
DATA.loadJson('anims_s', 'animations_server.json')
DATA.loadJson('anims_c', 'animations_client.json')
DATA.loadJson('layers_s', 'sprite_layers_server.json')
DATA.loadJson('layers_c', 'sprite_layers_client.json')
DATA.loadJson('graphics', 'sprite_graphics_client.json')
DATA.loadImage('img', 'sprites0.png')


function empty_element(elt) {
    while (elt.lastElementChild != null) {
        elt.removeChild(elt.lastElementChild);
    }
}




var ASM = init_asm();

var canvas = document.getElementById('cnv-main');
var ctx = canvas.getContext('2d');
ctx.imageSmoothingEnabled = false;
ctx.mozImageSmoothingEnabled = false;
console.log(ctx);
ASM.update();
asm_draw(ctx, ASM);


var frame_pending = false;

function frame() {
    frame_pending = false;
    ASM.update();
    ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
    asm_draw(ctx, ASM);
}

function request_frame() {
    if (!frame_pending) {
        frame_pending = true;
        window.requestAnimationFrame(frame);
    }
}


canvas.addEventListener('mousemove', function(evt) {
    ASM.handle_mouse_move(evt.offsetX, evt.offsetY);
    request_frame();
});

canvas.addEventListener('mousedown', function(evt) {
    ASM.handle_mouse_down(evt.offsetX, evt.offsetY);
    request_frame();
});

canvas.addEventListener('mouseup', function(evt) {
    ASM.handle_mouse_up(evt.offsetX, evt.offsetY);
    request_frame();
});


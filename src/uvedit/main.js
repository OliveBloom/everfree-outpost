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


var asm = init_asm();
window.ASM = asm;

var canvas = document.getElementById('cnv-main');
var ctx = canvas.getContext('2d');
ctx.imageSmoothingEnabled = false;
ctx.mozImageSmoothingEnabled = false;
console.log(ctx);
asm.update();
asm_draw(ctx, asm);


var frame_pending = false;

function frame() {
    frame_pending = false;
    asm.update();
    ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
    asm_draw(ctx, asm);
}

function request_frame() {
    if (!frame_pending) {
        frame_pending = true;
        window.requestAnimationFrame(frame);
    }
}


canvas.addEventListener('mousemove', function(evt) {
    asm.handle_mouse_move(evt.offsetX, evt.offsetY);
    request_frame();
});

canvas.addEventListener('mousedown', function(evt) {
    asm.handle_mouse_down(evt.offsetX, evt.offsetY);
    request_frame();
});

canvas.addEventListener('mouseup', function(evt) {
    asm.handle_mouse_up(evt.offsetX, evt.offsetY);
    request_frame();
});


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
    asm.init();
    return asm
}


window.ASM = init_asm();


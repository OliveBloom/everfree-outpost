var asmlibs_code_raw = function(global, env, buffer) {
    'use asm';

    var HEAP8 = new global.Int8Array(buffer);
    var HEAP16 = new global.Int16Array(buffer);
    var HEAP32 = new global.Int32Array(buffer);
    var HEAPU8 = new global.Uint8Array(buffer);
    var HEAPU16 = new global.Uint16Array(buffer);
    var HEAPU32 = new global.Uint32Array(buffer);
    var HEAPF32 = new global.Float32Array(buffer);
    var HEAPF64 = new global.Float64Array(buffer);

    var STACKTOP = env.STACK_START|0;
    var STACK_MAX = env.STACK_END|0;

    var abort = env.abort;
    var _llvm_trap = env.abort;
    var _write_str = env.writeStr;
    var _flush_str = env.flushStr;
    var _flush_str_warn = env.flushStrWarn;
    var _flush_str_err = env.flushStrErr;
    var _now = env.now;
    var Math_imul = global.Math.imul;
    var nan = global.NaN;
    var _emscripten_memcpy_big = env._emscripten_memcpy_big;

    var _asmgl_has_extension = env.asmgl_has_extension;

    var _asmgl_gen_buffer = env.asmgl_gen_buffer;
    var _asmgl_delete_buffer = env.asmgl_delete_buffer;
    var _asmgl_bind_buffer = env.asmgl_bind_buffer;
    var _asmgl_buffer_data_alloc = env.asmgl_buffer_data_alloc;
    var _asmgl_buffer_subdata = env.asmgl_buffer_subdata;

    var _asmgl_load_shader = env.asmgl_load_shader;
    var _asmgl_delete_shader = env.asmgl_delete_shader;
    var _asmgl_bind_shader = env.asmgl_bind_shader;
    var _asmgl_get_uniform_location = env.asmgl_get_uniform_location;
    var _asmgl_get_attrib_location = env.asmgl_get_attrib_location;
    var _asmgl_set_uniform_1i = env.asmgl_set_uniform_1i;
    var _asmgl_set_uniform_1f = env.asmgl_set_uniform_1f;
    var _asmgl_set_uniform_2f = env.asmgl_set_uniform_2f;
    var _asmgl_set_uniform_3f = env.asmgl_set_uniform_3f;
    var _asmgl_set_uniform_4f = env.asmgl_set_uniform_4f;

    var _asmgl_load_texture = env.asmgl_load_texture;
    var _asmgl_gen_texture = env.asmgl_gen_texture;
    var _asmgl_delete_texture = env.asmgl_delete_texture;
    var _asmgl_active_texture = env.asmgl_active_texture;
    var _asmgl_bind_texture = env.asmgl_bind_texture;
    var _asmgl_texture_image = env.asmgl_texture_image;
    var _asmgl_texture_subimage = env.asmgl_texture_subimage;

    var _asmgl_gen_framebuffer = env.asmgl_gen_framebuffer;
    var _asmgl_delete_framebuffer = env.asmgl_delete_framebuffer;
    var _asmgl_bind_framebuffer = env.asmgl_bind_framebuffer;
    var _asmgl_gen_renderbuffer = env.asmgl_gen_renderbuffer;
    var _asmgl_delete_renderbuffer = env.asmgl_delete_renderbuffer;
    var _asmgl_framebuffer_texture = env.asmgl_framebuffer_texture;
    var _asmgl_framebuffer_renderbuffer = env.asmgl_framebuffer_renderbuffer;
    var _asmgl_check_framebuffer_status = env.asmgl_check_framebuffer_status;
    var _asmgl_draw_buffers = env.asmgl_draw_buffers;

    var _asmgl_viewport = env.asmgl_viewport;
    var _asmgl_clear_color = env.asmgl_clear_color;
    var _asmgl_clear_depth = env.asmgl_clear_depth;
    var _asmgl_clear = env.asmgl_clear;
    var _asmgl_set_depth_test = env.asmgl_set_depth_test;
    var _asmgl_set_blend_mode = env.asmgl_set_blend_mode;
    var _asmgl_enable_vertex_attrib_array = env.asmgl_enable_vertex_attrib_array;
    var _asmgl_disable_vertex_attrib_array = env.asmgl_disable_vertex_attrib_array;
    var _asmgl_vertex_attrib_pointer = env.asmgl_vertex_attrib_pointer;
    var _asmgl_draw_arrays_triangles = env.asmgl_draw_arrays_triangles;

    var _ap_config_get = env.ap_config_get;
    var _ap_config_get_int = env.ap_config_get_int;
    var _ap_config_set = env.ap_config_set;
    var _ap_config_clear = env.ap_config_clear;
    var _ap_set_cursor = env.ap_set_cursor;
    var _ap_send_move_item = env.ap_send_move_item;

    var tempRet0 = 0;

    function __adjust_stack(offset) {
        offset = offset|0;
        STACKTOP = STACKTOP + offset|0;
        if ((STACKTOP|0) >= (STACK_MAX|0)) abort();
        return (STACKTOP - offset)|0;
    }


    function _bitshift64Lshr(low, high, bits) {
        low = low|0; high = high|0; bits = bits|0;
        var ander = 0;
        if ((bits|0) < 32) {
            ander = ((1 << bits) - 1)|0;
            tempRet0 = high >>> bits;
            return (low >>> bits) | ((high&ander) << (32 - bits));
        }
        tempRet0 = 0;
        return (high >>> (bits - 32))|0;
    }

    function _bitshift64Shl(low, high, bits) {
        low = low|0; high = high|0; bits = bits|0;
        var ander = 0;
        if ((bits|0) < 32) {
            ander = ((1 << bits) - 1)|0;
            tempRet0 = (high << bits) | ((low&(ander << (32 - bits))) >>> (32 - bits));
            return low << bits;
        }
        tempRet0 = low << (bits - 32);
        return 0;
    }

    function _memset(ptr, value, num) {
        ptr = ptr|0; value = value|0; num = num|0;
        var stop = 0, value4 = 0, stop4 = 0, unaligned = 0;
        stop = (ptr + num)|0;
        if ((num|0) >= 20) {
            // This is unaligned, but quite large, so work hard to get to aligned settings
            value = value & 0xff;
            unaligned = ptr & 3;
            value4 = value | (value << 8) | (value << 16) | (value << 24);
            stop4 = stop & ~3;
            if (unaligned) {
                unaligned = (ptr + 4 - unaligned)|0;
                while ((ptr|0) < (unaligned|0)) { // no need to check for stop, since we have large num
                    HEAP8[((ptr)>>0)]=value;
                    ptr = (ptr+1)|0;
                }
            }
            while ((ptr|0) < (stop4|0)) {
                HEAP32[((ptr)>>2)]=value4;
                ptr = (ptr+4)|0;
            }
        }
        while ((ptr|0) < (stop|0)) {
            HEAP8[((ptr)>>0)]=value;
            ptr = (ptr+1)|0;
        }
        return (ptr-num)|0;
    }

    function _memmove(dest, src, num) {
        dest = dest|0; src = src|0; num = num|0;
        var ret = 0;
        if (((src|0) < (dest|0)) & ((dest|0) < ((src + num)|0))) {
            // Unlikely case: Copy backwards in a safe manner
            ret = dest;
            src = (src + num)|0;
            dest = (dest + num)|0;
            while ((num|0) > 0) {
                dest = (dest - 1)|0;
                src = (src - 1)|0;
                num = (num - 1)|0;
                HEAP8[((dest)>>0)]=((HEAP8[((src)>>0)])|0);
            }
            dest = ret;
        } else {
            _memcpy(dest, src, num) | 0;
        }
        return dest | 0;
    }

    function _memcpy(dest, src, num) {
        dest = dest|0; src = src|0; num = num|0;
        var ret = 0;
        if ((num|0) >= 4096) return _emscripten_memcpy_big(dest|0, src|0, num|0)|0;
        ret = dest|0;
        if ((dest&3) == (src&3)) {
            while (dest & 3) {
                if ((num|0) == 0) return ret|0;
                HEAP8[((dest)>>0)]=((HEAP8[((src)>>0)])|0);
                dest = (dest+1)|0;
                src = (src+1)|0;
                num = (num-1)|0;
            }
            while ((num|0) >= 4) {
                HEAP32[((dest)>>2)]=((HEAP32[((src)>>2)])|0);
                dest = (dest+4)|0;
                src = (src+4)|0;
                num = (num-4)|0;
            }
        }
        while ((num|0) > 0) {
            HEAP8[((dest)>>0)]=((HEAP8[((src)>>0)])|0);
            dest = (dest+1)|0;
            src = (src+1)|0;
            num = (num-1)|0;
        }
        return ret|0;
    }

    function _memcmp(a, b, num) {
        a = a|0; b = b|0; num = num|0;
        var a_byte = 0;
        var b_byte = 0;
        if ((a&3) == (b&3)) {
            while (b & 3) {
                if ((num|0) == 0) return 0;
                a_byte = HEAP8[a>>0]|0;
                b_byte = HEAP8[b>>0]|0;
                if ((a_byte|0) != (b_byte|0)) {
                    return (b_byte - a_byte)|0;
                }
                a = (a+1)|0;
                b = (b+1)|0;
                num = (num-1)|0;
            }
            while ((num|0) >= 4) {
                if ((HEAP32[a>>2]|0) != (HEAP32[b>>2]|0)) {
                    // Let the later case take care of it.
                    break;
                }
                a = (a+4)|0;
                b = (b+4)|0;
                num = (num-4)|0;
            }
        }
        while ((num|0) > 0) {
            a_byte = HEAP8[a>>0]|0;
            b_byte = HEAP8[b>>0]|0;
            if ((a_byte|0) != (b_byte|0)) {
                return (b_byte - a_byte)|0;
            }
            a = (a+1)|0;
            b = (b+1)|0;
            num = (num-1)|0;
        }
        return 0;
    }

    function _llvm_cttz_i32(x) {
        x = x|0;
        var y = 0;
        var n = 31;

        if ((x|0) == 0) {
            return 32;
        }

        y = (x << 16)|0; if ((y|0) != 0) { n = (n - 16)|0; x = (y|0); }
        y = (x <<  8)|0; if ((y|0) != 0) { n = (n -  8)|0; x = (y|0); }
        y = (x <<  4)|0; if ((y|0) != 0) { n = (n -  4)|0; x = (y|0); }
        y = (x <<  2)|0; if ((y|0) != 0) { n = (n -  2)|0; x = (y|0); }
        y = (x <<  1)|0; if ((y|0) != 0) { n = (n -  1)|0; x = (y|0); }
        return (n|0);
    }

    // INSERT_EMSCRIPTEN_FUNCTIONS

    return ({
        __adjust_stack: __adjust_stack,

        // INSERT_EMSCRIPTEN_EXPORTS
    });
};

window.asmlibs_code = function(global, env, buffer) {
    var heap_u8 = new Uint8Array(buffer);

    env._emscripten_memcpy_big = function(dest, src, num) {
        heap_u8.set(heap_u8.subarray(src, src+num), dest);
        return dest;
    };

    return asmlibs_code_raw(global, env, buffer);
};

window.asmlibs_data = new Uint8Array([
    // INSERT_EMSCRIPTEN_STATIC
]);

window.asmlibs_data_size =
    // INSERT_EMSCRIPTEN_DATA_SIZE
;

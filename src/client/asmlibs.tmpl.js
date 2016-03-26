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
    var _now = env.now;
    var Math_imul = global.Math.imul;
    var _emscripten_memcpy_big = env._emscripten_memcpy_big;

    var _asmgl_gen_buffer = env.asmgl_gen_buffer;
    var _asmgl_delete_buffer = env.asmgl_delete_buffer;
    var _asmgl_bind_buffer_array = env.asmgl_bind_buffer_array;
    var _asmgl_bind_buffer_index = env.asmgl_bind_buffer_index;
    var _asmgl_buffer_data_alloc = env.asmgl_buffer_data_alloc;
    var _asmgl_buffer_subdata = env.asmgl_buffer_subdata;

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
        get_sizes: _get_sizes,

        asmmalloc_init: _asmmalloc_init,
        asmmalloc_reinit: _asmmalloc_reinit,
        asmmalloc_max_allocated_address: _asmmalloc_max_allocated_address,
        asmmalloc_alloc: _asmmalloc_alloc,
        asmmalloc_free: _asmmalloc_free,
        asmmalloc_debug_print: _asmmalloc_debug_print,

        asmlibs_init: _asmlibs_init,
        data_init: _data_init,
        client_init: _client_init,
        client_reset: _client_reset,

        load_terrain_chunk: _load_terrain_chunk,

        structure_appear: _structure_appear,
        structure_gone: _structure_gone,
        structure_replace: _structure_replace,


        collide: _collide,
        find_ceiling: _find_ceiling,
        floodfill: _floodfill,

        prepare_geometry: _prepare_geometry,
        get_terrain_geometry_buffer: _get_terrain_geometry_buffer,
        get_structure_geometry_buffer: _get_structure_geometry_buffer,
        get_light_geometry_buffer: _get_light_geometry_buffer,
        get_ui_geometry_buffer: _get_ui_geometry_buffer,

        client_bench: _client_bench,
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

window.asmlibs_data = new Uint8Array(
    // INSERT_EMSCRIPTEN_STATIC
);

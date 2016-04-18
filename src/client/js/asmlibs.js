var module = window['asmlibs_code'];
var static_data = window['asmlibs_data'];
var static_size = window['asmlibs_data_size'];

var config = require('config');
var Vec = require('util/vec').Vec;
var decodeUtf8 = require('util/misc').decodeUtf8;
var encodeUtf8 = require('util/misc').encodeUtf8;

var AsmGl = require('asmgl').AsmGl;


// Memory layout

// Emscripten puts the first static at address 8 to  avoid storing data at
// address 0.
var STATIC_START = 8;  
var STATIC_SIZE = static_size;
var STATIC_END = STATIC_START + STATIC_SIZE;

// Align STACK_START to an 8-byte boundary.
var STACK_START = (STATIC_END + 7) & ~7;
// Give at least 16k for stack, and align to 4k boundary.
var STACK_END = (STACK_START + 0x4000 + 0x0fff) & ~0x0fff;
var STACK_SIZE = STACK_END - STACK_START;
console.assert(STACK_SIZE >= 0x1000, 'want at least 4kb for stack');

var HEAP_START = STACK_END;


// External functions

var module_env = function(asm) {
    var msg_buffer = '';

    return ({
        'abort': function() {
            console.assert(false, 'abort');
            throw 'abort';
        },

        'writeStr': function(ptr, len) {
            var view = asm._makeView(Uint8Array, ptr, len);
            msg_buffer += decodeUtf8(view);
        },

        'flushStr': function() {
            console.log(msg_buffer);
            msg_buffer = '';
        },

        'now': function() {
            return Date.now() & 0xffffffff;
        },


        'asmgl_gen_buffer': function() {
            return asm.asmgl.genBuffer();
        },
        'asmgl_delete_buffer': function(name) {
            asm.asmgl.deleteBuffer(name);
        },
        'asmgl_bind_buffer': function(target_idx, name) {
            asm.asmgl.bindBuffer(target_idx, name);
        },
        'asmgl_bind_buffer_index': function(name) {
            asm.asmgl.bindBufferIndex(name);
        },
        'asmgl_buffer_data_alloc': function(target, len) {
            asm.asmgl.bufferDataAlloc(target, len);
        },
        'asmgl_buffer_subdata': function(target, offset, ptr, len) {
            var data = asm._makeView(Uint8Array, ptr, len);
            asm.asmgl.bufferSubdata(target, offset, data);
        },

        'asmgl_load_shader': function(
                vert_name_ptr, vert_name_len,
                frag_name_ptr, frag_name_len,
                defs_ptr, defs_len) {
            var vert_name = asm._loadString(vert_name_ptr, vert_name_len);
            var frag_name = asm._loadString(frag_name_ptr, frag_name_len);
            var defs = asm._loadString(defs_ptr, defs_len);
            return asm.asmgl.loadShader(vert_name, frag_name, defs);
        },
        'asmgl_delete_shader': function(name) {
            asm.asmgl.deleteShader(name);
        },
        'asmgl_bind_shader': function(name) {
            asm.asmgl.bindShader(name);
        },
        'asmgl_get_uniform_location': function(shader_name, name_ptr, name_len) {
            var var_name = asm._loadString(name_ptr, name_len);
            return asm.asmgl.getUniformLocation(shader_name, var_name);
        },
        'asmgl_get_attrib_location': function(shader_name, name_ptr, name_len) {
            var var_name = asm._loadString(name_ptr, name_len);
            return asm.asmgl.getAttribLocation(shader_name, var_name);
        },
        'asmgl_set_uniform_1i': function(loc, value) {
            asm.asmgl.setUniform1i(loc, value);
        },
        'asmgl_set_uniform_1f': function(loc, value) {
            asm.asmgl.setUniform1f(loc, value);
        },
        'asmgl_set_uniform_2f': function(loc, ptr) {
            var view = asm._makeView(Float32Array, ptr, 8);
            asm.asmgl.setUniform2f(loc, view);
        },
        'asmgl_set_uniform_3f': function(loc, ptr) {
            var view = asm._makeView(Float32Array, ptr, 12);
            asm.asmgl.setUniform3f(loc, view);
        },
        'asmgl_set_uniform_4f': function(loc, ptr) {
            var view = asm._makeView(Float32Array, ptr, 16);
            asm.asmgl.setUniform4f(loc, view);
        },

        'asmgl_load_texture': function(name_ptr, name_len, size_ptr) {
            var name = asm._loadString(name_ptr, name_len);
            var size_view = asm._makeView(Uint16Array, size_ptr, 4);
            return asm.asmgl.loadTexture(name, size_view);
        },
        'asmgl_gen_texture': function(width, height, is_depth) {
            return asm.asmgl.genTexture(width, height, is_depth);
        },
        'asmgl_delete_texture': function(name) {
            asm.asmgl.deleteTexture(name);
        },
        'asmgl_active_texture': function(unit) {
            asm.asmgl.activeTexture(unit);
        },
        'asmgl_bind_texture': function(name) {
            asm.asmgl.bindTexture(name);
        },

        'asmgl_gen_framebuffer': function() {
            return asm.asmgl.genFramebuffer();
        },
        'asmgl_delete_framebuffer': function(name) {
            asm.asmgl.deleteFramebuffer(name);
        },
        'asmgl_bind_framebuffer': function(name) {
            asm.asmgl.bindFramebuffer(name);
        },
        'asmgl_gen_renderbuffer': function(width, height, is_depth) {
            return asm.asmgl.genRenderbuffer(width, height, is_depth);
        },
        'asmgl_delete_renderbuffer': function(name) {
            asm.asmgl.deleteRenderbuffer(name);
        },
        'asmgl_framebuffer_texture': function(tex_name, attachment) {
            asm.asmgl.framebufferTexture(tex_name, attachment);
        },
        'asmgl_framebuffer_renderbuffer': function(rb_name, attachment) {
            asm.asmgl.framebufferRenderbuffer(rb_name, attachment);
        },
        'asmgl_check_framebuffer_status': function() {
            return asm.asmgl.checkFramebufferStatus();
        },
        'asmgl_draw_buffers': function(num_attachments) {
            asm.asmgl.drawBuffers(num_attachments);
        },

        'asmgl_viewport': function(x, y, w, h) {
            asm.asmgl.viewport(x, y, w, h);
        },
        'asmgl_clear_color': function(r, g, b, a) {
            asm.asmgl.clearColor(r, g, b, a);
        },
        'asmgl_clear_depth': function(d) {
            asm.asmgl.clearDepth(d);
        },
        'asmgl_clear': function() {
            asm.asmgl.clear();
        },
        'asmgl_set_depth_test': function(enable) {
            asm.asmgl.setDepthTest(enable);
        },
        'asmgl_enable_vertex_attrib_array': function(index) {
            asm.asmgl.enableVertexAttribArray(index);
        },
        'asmgl_disable_vertex_attrib_array': function(index) {
            asm.asmgl.disableVertexAttribArray(index);
        },
        'asmgl_vertex_attrib_pointer': function(loc, count, ty, normalize, stride, offset) {
            asm.asmgl.vertexAttribPointer(loc, count, ty, normalize, stride, offset);
        },
        'asmgl_draw_arrays_triangles': function(start, count) {
            asm.asmgl.drawArraysTriangles(start, count);
        },

        'ap_config_get': function(key_ptr, key_len, value_len_p) {
            var key_view = asm._makeView(Uint8Array, key_ptr, key_len);
            var key = decodeUtf8(key_view);
            var value = config.rawGet(key);

            var value_utf8 = unescape(encodeURIComponent('' + value));
            var len = value_utf8.length;
            var value_view = asm._heapAlloc(Uint8Array, len);
            for (var i = 0; i < len; ++i) {
                value_view[i] = value_utf8.charCodeAt(i);
            }

            var value_len_p_view = asm._makeView(Uint32Array, value_len_p, 4);
            value_len_p_view[0] = len;
            return value_view.byteOffset;
        },

        'ap_config_set': function(key_ptr, key_len, value_ptr, value_len) {
            var key_view = asm._makeView(Uint8Array, key_ptr, key_len);
            var key = decodeUtf8(key_view);
            var value_view = asm._makeView(Uint8Array, value_ptr, value_len);
            var value = decodeUtf8(value_view);

            config.rawSet(key, value);
        },

        'ap_config_clear': function(key_ptr, key_len) {
            var key_view = asm._makeView(Uint8Array, key_ptr, key_len);
            var key = decodeUtf8(key_view);

            config.rawClear(key);
        },


        'STACK_START': STACK_START,
        'STACK_END': STACK_END,
    });
};


// Helper functions

function memcpy(dest_buffer, dest_offset, src_buffer, src_offset, len) {
    var dest = new Int8Array(dest_buffer, dest_offset, len);
    var src = new Int8Array(src_buffer, src_offset, len);
    dest.set(src);
}

function next_heap_size(size) {
    // "the heap object's byteLength must be either 2^n for n in [12, 24) or
    // 2^24 * n for n >= 1"
    if (size <= (1 << 12)) {
        return (1 << 12);
    } else if (size >= (1 << 24)) {
        return (size | ((1 << 24) - 1)) + 1;
    } else {
        for (var i = 12 + 1; i < 24; ++i) {
            if (size <= (1 << i)) {
                return (1 << i);
            }
        }
        console.assert(false, 'failed to compute next heap size for', size);
        return (1 << 24);
    }
}

function store_vec(view, offset, vec) {
    view[offset + 0] = vec.x;
    view[offset + 1] = vec.y;
    view[offset + 2] = vec.z;
}

var INIT_HEAP_SIZE = 4 * 1024 * 1024;
var HEAP_PADDING = 256 * 1024;

/** @constructor */
function DynAsm() {
    this.asmgl = new AsmGl();

    this.buffer = new ArrayBuffer(next_heap_size(INIT_HEAP_SIZE));
    this._memcpy(STATIC_START, static_data);
    this._raw = module(window, module_env(this), this.buffer);

    this._raw['asmlibs_init']();
    this._raw['asmmalloc_init'](HEAP_START, this.buffer.byteLength);

    this.data = null;
    this.client = null;
    this.SIZEOF = this._calcSizeof();

    // TODO: hack (also see shaders.js)
    window.SIZEOF = this.SIZEOF;
}
exports.DynAsm = DynAsm;

DynAsm.prototype._grow = function(size) {
    var old_buffer = this.buffer;

    this.buffer = new ArrayBuffer(next_heap_size(size));
    this._memcpy(0, old_buffer);
    this._raw = module(window, module_env(this.buffer), this.buffer);

    this._raw['asmmalloc_reinit'](this.buffer.byteLength);
};

DynAsm.prototype._heapCheck = function() {
    var max = this._raw['asmmalloc_max_allocated_address'];
    if (max >= this.buffer.length - HEAP_PADDING) {
        this._grow(this.buffer.byteLength * 2);
    }
};

DynAsm.prototype._calcSizeof = function() {
    var EXPECT_SIZES = 10;
    var sizes = this._stackAlloc(Int32Array, EXPECT_SIZES);

    var num_sizes = this._raw['get_sizes'](sizes.byteOffset);
    console.assert(num_sizes == EXPECT_SIZES,
            'expected sizes for ' + EXPECT_SIZES + ' types, but got ' + num_sizes);

    var index = 0;
    var next = function() { return sizes[index++]; };
    var result = {};

    result.Client = next();
    result.ClientAlignment = next();
    result.Data = next();
    result.DataAlignment = next();

    result.TerrainVertex = next();
    result.StructureVertex = next();
    result.LightVertex = next();
    result.UIVertex = next();

    result.Scene = next();

    result.Item = next();

    console.assert(index == EXPECT_SIZES,
            'some items were left over after building sizeof', index, EXPECT_SIZES);

    return result;
};

DynAsm.prototype._stackAlloc = function(type, count) {
    var size = count * type.BYTES_PER_ELEMENT;
    var base = this._raw['__adjust_stack']((size + 7) & ~7);
    return new type(this.buffer, base, count);
};

DynAsm.prototype._stackFree = function(view) {
    var size = view.byteLength;
    this._raw['__adjust_stack'](-((size + 7) & ~7));
};

DynAsm.prototype._heapAlloc = function(type, count) {
    var size = count * type.BYTES_PER_ELEMENT;
    var base = this._raw['asmmalloc_alloc'](size, type.BYTES_PER_ELEMENT);
    return new type(this.buffer, base, count);
};

DynAsm.prototype._heapFree = function(view) {
    this._raw['asmmalloc_free'](view.byteOffset);
};

DynAsm.prototype._makeView = function(type, offset, bytes) {
    return new type(this.buffer, offset, bytes / type.BYTES_PER_ELEMENT);
};

DynAsm.prototype._memcpy = function(dest_offset, data) {
    if (data.constructor !== ArrayBuffer) {
        memcpy(this.buffer, dest_offset, data.buffer, data.byteOffset, data.byteLength);
    } else {
        memcpy(this.buffer, dest_offset, data, 0, data.byteLength);
    }
};

DynAsm.prototype._loadString = function(ptr, len) {
    var view = this._makeView(Uint8Array, ptr, len);
    return decodeUtf8(view);
};

DynAsm.prototype._allocString = function(s) {
    var utf8 = unescape(encodeURIComponent('' + s));
    var len = utf8.length;
    var view = this._heapAlloc(Uint8Array, len);
    for (var i = 0; i < len; ++i) {
        view[i] = utf8.charCodeAt(i);
    }
    return view;
};

DynAsm.prototype.initClient = function(gl, assets) {
    // AsmGl must be initialized before calling `client_init`.
    this.asmgl.init(gl, assets);

    var blobs = this._stackAlloc(Int32Array, 11 * 2);
    var idx = 0;
    var this_ = this;
    var load_blob = function(x) {
        var len = x.byteLength;
        var addr = this_._raw['asmmalloc_alloc'](len, 4);
        this_._memcpy(addr, x);
        blobs[idx++] = addr;
        blobs[idx++] = len;
    };

    load_blob(assets['block_defs_bin']);
    load_blob(assets['item_defs_bin']);
    load_blob(assets['item_strs_bin']);
    load_blob(assets['template_defs_bin']);
    load_blob(assets['template_part_defs_bin']);
    load_blob(assets['template_vert_defs_bin']);
    load_blob(assets['template_shape_defs_bin']);
    load_blob(assets['animation_defs_bin']);
    load_blob(assets['sprite_layer_defs_bin']);
    load_blob(assets['sprite_graphics_defs_bin']);
    load_blob(assets['pony_layer_table_bin']);

    // Item names get custom handling


    this.data = this._raw['asmmalloc_alloc'](this.SIZEOF.Data, this.SIZEOF.DataAlignment);
    // NB: takes ownership of `blobs`
    this._raw['data_init'](blobs.byteOffset, this.data);

    this.client = this._raw['asmmalloc_alloc'](this.SIZEOF.Client, this.SIZEOF.ClientAlignment);
    this._raw['client_init'](this.data, this.client);

    this._stackFree(blobs);
};

DynAsm.prototype.resetClient = function() {
    this._raw['client_reset'](this.client);
};

DynAsm.prototype.structureAppear = function(id, x, y, z, template_id, oneshot_start) {
    this._raw['structure_appear'](this.client,
            id, x, y, z, template_id, oneshot_start);
};

DynAsm.prototype.structureGone = function(id) {
    this._raw['structure_gone'](this.client, id);
};

DynAsm.prototype.structureReplace = function(id, template_id, oneshot_start) {
    this._raw['structure_replace'](this.client, id, template_id, oneshot_start);
};

DynAsm.prototype.entityAppear = function(id, appearance, name) {
    var name_ptr = 0;
    var name_len = 0;
    if (name != null && name.length > 0) {
        var name_view = this._allocString(name);
        name_ptr = name_view.byteOffset;
        name_len = name_view.byteLength;
    }

    // Library takes ownership of the name allocation.
    this._raw['entity_appear'](this.client, id, appearance, name_ptr, name_len);
};

DynAsm.prototype.entityGone = function(id) {
    this._raw['entity_gone'](this.client, id);
};

DynAsm.prototype.entityUpdate = function(id, motion, anim) {
    var arr = this._stackAlloc(Int32Array, 9);

    arr[0] = motion.start_pos.x;
    arr[1] = motion.start_pos.y;
    arr[2] = motion.start_pos.z;
    arr[3] = motion.end_pos.x;
    arr[4] = motion.end_pos.y;
    arr[5] = motion.end_pos.z;

    arr[6] = motion.start_time;
    arr[7] = motion.end_time;
    arr[8] = anim;

    this._raw['entity_update'](this.client, id, motion.start_time, arr.byteOffset);

    this._stackFree(arr);
};

DynAsm.prototype.inventoryAppear = function(id, items) {
    var item_arr = this._heapAlloc(Uint8Array, items.length * this.SIZEOF.Item);
    var view = new DataView(item_arr.buffer, item_arr.byteOffset, item_arr.byteLength);

    for (var i = 0; i < items.length; ++i) {
        var base = i * this.SIZEOF.Item;
        var item = items[i];
        view.setUint16( base +  0,  item.item_id, true);
        view.setUint8(  base +  2,  item.count);
    }

    // Takes ownership of `items`.
    this._raw['inventory_appear'](this.client, id, item_arr.byteOffset, item_arr.byteLength);
};

DynAsm.prototype.inventoryGone = function(id) {
    this._raw['inventory_gone'](this.client, id);
};

DynAsm.prototype.inventoryUpdate = function(id, slot, item) {
    this._raw['inventory_update'](this.client, id, slot, item.item_id, item.count);
};

DynAsm.prototype.inventoryMainId = function(id) {
    this._raw['inventory_main_id'](this.client, id);
};

DynAsm.prototype.inventoryAbilityId = function(id) {
    this._raw['inventory_ability_id'](this.client, id);
};

DynAsm.prototype.inputKey = function(code) {
    return this._raw['input_key'](this.client, code);
};

DynAsm.prototype.inputMouseMove = function(x, y) {
    return this._raw['input_mouse_move'](this.client, x, y);
};

DynAsm.prototype.openInventoryDialog = function() {
    return this._raw['open_inventory_dialog'](this.client);
};

DynAsm.prototype.setRegionShape = function(base, size, layer, shape) {
    var region = this._stackAlloc(Int32Array, 6);
    store_vec(region, 0, base);
    store_vec(region, 3, size);

    var buf = this._heapAlloc(Uint8Array, shape.length);
    buf.set(shape);
    this._raw['set_region_shape'](this.client,
            region.byteOffset, layer + 1, buf.byteOffset, buf.byteLength);
    this._heapFree(buf);

    this._stackFree(region);
};

DynAsm.prototype.clearRegionShape = function(base, size, layer) {
    var region = this._stackAlloc(Int32Array, 6);
    store_vec(region, 0, base);
    store_vec(region, 3, size);

    var buf = this._heapAlloc(Uint8Array, size.x * size.y * size.z);
    buf.fill(0);
    this._raw['set_region_shape'](this.client,
            region.byteOffset, layer + 1, buf.byteOffset, buf.byteLength);
    this._heapFree(buf);

    this._stackFree(region);
};

DynAsm.prototype.collide = function(pos, size, velocity) {
    var input = this._stackAlloc(Int32Array, 9);
    var output = this._stackAlloc(Int32Array, 4);

    store_vec(input, 0, pos);
    store_vec(input, 3, size);
    store_vec(input, 6, velocity);

    this._raw['collide'](this.client, input.byteOffset, output.byteOffset);

    var result = ({
        x: output[0],
        y: output[1],
        z: output[2],
        t: output[3],
    });

    this._stackFree(output);
    this._stackFree(input);

    return result;
};

DynAsm.prototype.findCeiling = function(pos) {
    var vec = this._stackAlloc(Int32Array, 3);
    store_vec(vec, 0, pos);

    var result = this._raw['find_ceiling'](this.client, vec.byteOffset);

    this._stackFree(vec);

    return result;
};

DynAsm.prototype.floodfill = function(pos, radius) {
    var size = radius * 2;
    var len = size * size;

    var pos_buf = this._stackAlloc(Int32Array, 3);
    store_vec(pos_buf, 0, pos);
    var grid = this._stackAlloc(Uint8Array, len);
    grid.fill(0);
    var queue = this._stackAlloc(Uint8Array, 2 * len);

    this._raw['floodfill'](
            this.client,
            pos_buf.byteOffset, radius,
            grid.byteOffset, len,
            queue.byteOffset, 2 * len);

    var result = grid.slice();

    this._stackFree(queue);
    this._stackFree(grid);
    this._stackFree(pos_buf);

    return result;
};

DynAsm.prototype.loadTerrainChunk = function(cx, cy, data) {
    var buf = this._heapAlloc(Uint16Array, data.length);
    buf.set(data);
    this._raw['load_terrain_chunk'](this.client,
            cx, cy, buf.byteOffset, buf.byteLength);
    this._heapFree(buf);
};

DynAsm.prototype.renderFrame = function(scene) {
    var f32 = this._stackAlloc(Float32Array, this.SIZEOF.Scene / 4);
    var i32 = new Int32Array(f32.buffer, f32.byteOffset, f32.length);

    i32[ 0] = scene.canvas_size[0];
    i32[ 1] = scene.canvas_size[1];
    i32[ 2] = scene.camera_pos[0];
    i32[ 3] = scene.camera_pos[1];
    i32[ 4] = scene.camera_size[0];
    i32[ 5] = scene.camera_size[1];
    i32[ 6] = scene.now;

    f32[ 7] = scene.camera_pos[0];
    f32[ 8] = scene.camera_pos[1];
    f32[ 9] = scene.camera_size[0];
    f32[10] = scene.camera_size[1];
    f32[11] = scene.slice_center[0];
    f32[12] = scene.slice_center[1];
    f32[13] = scene.slice_z;
    f32[14] = scene.now;

    this._raw['render_frame'](this.client, f32.byteOffset);

    this._stackFree(f32);
};

DynAsm.prototype.bench = function() {
    return this._raw['client_bench'](this.client);
};


function AsmClientInput(asm) {
    this._asm = asm;
}
exports.AsmClientInput = AsmClientInput;

AsmClientInput.prototype.handleMouseMove = function(evt) {
    var ret = this._asm.inputMouseMove(evt.x, evt.y);
    if (!ret) {
        evt.forward();
    }
    return ret;
};

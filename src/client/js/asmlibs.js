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

        'flushStrWarn': function() {
            console.warn(msg_buffer);
            msg_buffer = '';
        },

        'flushStrErr': function() {
            console.error(msg_buffer);
            window.onerror(msg_buffer, '<native code>', 0, 0, null);
            msg_buffer = '';
        },



        'asmgl_has_extension': function(name_ptr, name_len) {
            var name = asm._loadString(name_ptr, name_len);
            return asm.asmgl.hasExtension(name);
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
        'asmgl_gen_texture': function(width, height, kind) {
            return asm.asmgl.genTexture(width, height, kind);
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
        'asmgl_texture_image': function(width, height, kind, data_ptr, data_len) {
            var view = asm._makeView(Uint8Array, data_ptr, data_len);
            asm.asmgl.textureImage(width, height, kind, view);
        },
        'asmgl_texture_subimage': function(x, y, width, height, kind, data_ptr, data_len) {
            var view = asm._makeView(Uint8Array, data_ptr, data_len);
            asm.asmgl.textureSubimage(x, y, width, height, kind, view);
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
        'asmgl_set_blend_mode': function(mode) {
            asm.asmgl.setBlendMode(mode);
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
            var key = asm._loadString(key_ptr, key_len);
            var value = config.rawGet(key);

            var value_view = asm._allocString(value);

            var value_len_p_view = asm._makeView(Uint32Array, value_len_p, 4);
            value_len_p_view[0] = value_view.byteLength;
            return value_view.byteOffset;
        },

        'ap_config_get_int': function(key_ptr, key_len) {
            var key = asm._loadString(key_ptr, key_len);
            var value = config.rawGet(key);
            return value|0;
        },

        'ap_config_set': function(key_ptr, key_len, value_ptr, value_len) {
            var key = asm._loadString(key_ptr, key_len);
            var value = asm._loadString(value_ptr, value_len);

            config.rawSet(key, value);
        },

        'ap_config_clear': function(key_ptr, key_len) {
            var key = asm._loadString(key_ptr, key_len);

            config.rawClear(key);
        },

        'ap_set_cursor': function(cursor) {
            var str;
            switch (cursor) {
                case 0: str = 'auto'; break;
                case 1: str = 'grabbing'; break;
                case 2: str = 'not-allowed'; break;
                default: throw 'bad cursor value: ' + cursor;
            }
            document.body.style.cursor = str;
        },

        'ap_send_move_item': function(src_inv, src_slot, dest_inv, dest_slot, amount) {
            asm.conn.sendMoveItem(src_inv, src_slot, dest_inv, dest_slot, amount);
        },

        'ap_send_close_dialog': function() {
            asm.conn.sendCloseDialog();
        },

        'ap_get_time': function() {
            return (Date.now() - asm._time_base) & 0x7fffffff;
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

var INIT_HEAP_SIZE = 8 * 1024 * 1024;
var HEAP_PADDING = 256 * 1024;

/** @constructor */
function DynAsm() {
    this.asmgl = new AsmGl();
    this.conn = null;   // Will be set later, in main.js
    this._time_base = Date.now();


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

    this._stackFree(sizes);

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

    var blob = assets['client_data'];
    var len = blob.byteLength;
    var ptr = this._raw['asmmalloc_alloc'](len, 8);
    this._memcpy(ptr, blob);

    this.data = this._raw['asmmalloc_alloc'](this.SIZEOF.Data, this.SIZEOF.DataAlignment);
    // NB: takes ownership of `ptr`
    this._raw['data_init'](ptr, len, this.data);

    this.client = this._raw['asmmalloc_alloc'](this.SIZEOF.Client, this.SIZEOF.ClientAlignment);
    this._raw['client_init'](this.data, this.client);

    console.log(' -- CLIENT INIT -- ');
};

DynAsm.prototype.resetClient = function() {
    this._raw['client_reset'](this.client);
};

DynAsm.prototype.structureAppear = function(id, x, y, z, template_id) {
    this._raw['structure_appear'](this.client, id, x, y, z, template_id);
};

DynAsm.prototype.structureGone = function(id) {
    this._raw['structure_gone'](this.client, id);
};

DynAsm.prototype.structureReplace = function(id, template_id) {
    this._raw['structure_replace'](this.client, id, template_id);
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

DynAsm.prototype.entityGone = function(id, time) {
    this._raw['entity_gone'](this.client, id, time);
};

DynAsm.prototype.entityMotionStart = function(id, time, pos, velocity, anim) {
    this._raw['entity_motion_start'](this.client, id, time,
            pos.x, pos.y, pos.z,
            velocity.x, velocity.y, velocity.z,
            anim);
};

DynAsm.prototype.entityMotionEnd = function(id, time) {
    this._raw['entity_motion_end'](this.client, id, time);
};

DynAsm.prototype.entityActivityIcon = function(id, anim) {
    this._raw['entity_activity_icon'](this.client, id, anim);
};

DynAsm.prototype.setPawnId = function(entity_id) {
    this._raw['set_pawn_id'](this.client, entity_id);
};

DynAsm.prototype.setDefaultCameraPos = function(x, y, z) {
    this._raw['set_default_camera_pos'](this.client, x, y, z);
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

DynAsm.prototype.inputKey = function(code, shift) {
    return this._raw['input_key'](this.client, code, shift);
};

DynAsm.prototype.inputMouseMove = function(x, y) {
    return this._raw['input_mouse_move'](this.client, x, y);
};

DynAsm.prototype.inputMouseDown = function(x, y) {
    return this._raw['input_mouse_down'](this.client, x, y);
};

DynAsm.prototype.inputMouseUp = function(x, y) {
    return this._raw['input_mouse_up'](this.client, x, y);
};

DynAsm.prototype.openInventoryDialog = function() {
    return this._raw['open_inventory_dialog'](this.client);
};

DynAsm.prototype.openAbilityDialog = function() {
    return this._raw['open_ability_dialog'](this.client);
};

DynAsm.prototype.openContainerDialog = function(inv_id0, inv_id1) {
    return this._raw['open_container_dialog'](this.client, inv_id0, inv_id1);
};

DynAsm.prototype.closeDialog = function() {
    return this._raw['close_dialog'](this.client);
};

DynAsm.prototype.getActiveItem = function() {
    return this._raw['get_active_item'](this.client);
};

DynAsm.prototype.getActiveAbility = function() {
    return this._raw['get_active_ability'](this.client);
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

DynAsm.prototype.feedInput = function(time, bits) {
    this._raw['feed_input'](this.client, time, bits);
};

DynAsm.prototype.processedInputs = function(time, count) {
    this._raw['processed_inputs'](this.client, time, count);
};

DynAsm.prototype.activityChange = function(activity) {
    this._raw['activity_change'](this.client, activity);
};

DynAsm.prototype.loadTerrainChunk = function(cx, cy, data) {
    var buf = this._heapAlloc(Uint16Array, data.length);
    buf.set(data);
    this._raw['load_terrain_chunk'](this.client,
            cx, cy, buf.byteOffset, buf.byteLength);
    this._heapFree(buf);
};

DynAsm.prototype.renderFrame = function(ping) {
    this._raw['render_frame'](this.client, ping);
};

DynAsm.prototype.debugRecord = function(frame_time, ping) {
    this._raw['debug_record'](this.client, frame_time, ping);
};

DynAsm.prototype.initDayNight = function(time, base_offset, cycle_ms) {
    this._raw['init_day_night'](this.client, time, base_offset, cycle_ms);
};

DynAsm.prototype.setPlaneFlags = function(flags) {
    this._raw['set_plane_flags'](this.client, flags);
};

DynAsm.prototype.initTiming = function(server_now) {
    this._raw['init_timing'](this.client, server_now);
};

DynAsm.prototype.toggleCursor = function() {
    this._raw['toggle_cursor'](this.client);
};

DynAsm.prototype.calcScale = function(width, height) {
    return this._raw['calc_scale'](this.client, width, height);
};

DynAsm.prototype.resizeWindow = function(width, height) {
    this._raw['resize_window'](this.client, width, height);
};

DynAsm.prototype.ponyeditRender = function(app) {
    return this._raw['ponyedit_render'](this.client, app);
};

DynAsm.prototype.bench = function() {
    return this._raw['client_bench'](this.client);
};


function downloadArray(arr, name) {
    var b = new Blob([arr]);
    var url = window.URL.createObjectURL(b);

    var a = document.createElement('a');
    console.log(url);
    a.setAttribute('href', url);
    a.setAttribute('download', name);
    console.log('clicking...');
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
}

DynAsm.prototype.debugExport = function() {
    downloadArray(this.buffer, 'outpost_heap.dat');
};

DynAsm.prototype.debugImport = function() {
    var input = document.createElement('input');
    input.setAttribute('type', 'file');

    var this_ = this;

    input.onchange = function(evt) {
        if (input.files.length != 1) {
            return;
        }
        var f = input.files[0];
        var reader = new FileReader();
        reader.onloadend = function(evt) {
            var src = new Uint8Array(reader.result);
            var dest = new Uint8Array(this_.buffer);
            dest.set(src);

            window['ASMGL_LOG'] = true;

            this_._raw['client_reset_renderer'](this_.client);
            this_._raw['resize_window'](this_.client, window.innerWidth, window.innerHeight);
        };
        reader.readAsArrayBuffer(f);
    };

    document.body.appendChild(input);
    input.click();
    document.body.removeChild(input);
};


/** @constructor */
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

AsmClientInput.prototype.handleMouseDown = function(evt) {
    var ret = this._asm.inputMouseDown(evt.x, evt.y);
    if (!ret) {
        evt.forward();
    }
    return ret;
};

AsmClientInput.prototype.handleMouseUp = function(evt) {
    var ret = this._asm.inputMouseUp(evt.x, evt.y);
    if (!ret) {
        evt.forward();
    }
    return ret;
};

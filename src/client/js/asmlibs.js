var module = window['asmlibs_code'];
var static_data = window['asmlibs_data'];
var static_size = window['asmlibs_data_size'];

var config = require('config');
var Vec = require('util/vec').Vec;
var decodeUtf8 = require('util/misc').decodeUtf8;
var encodeUtf8 = require('util/misc').encodeUtf8;
var LOCAL_SIZE = require('data/chunk').LOCAL_SIZE;

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
        'asmgl_bind_buffer_array': function(name) {
            asm.asmgl.bindBufferArray(name);
        },
        'asmgl_bind_buffer_index': function(name) {
            asm.asmgl.bindBufferIndex(name);
        },
        'asmgl_buffer_data_alloc': function(len) {
            asm.asmgl.bufferDataAlloc(len);
        },
        'asmgl_buffer_subdata': function(offset, ptr, len) {
            asm.asmgl.bufferSubdata(offset, asm._makeView(Uint8Array, ptr, len));
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

INIT_HEAP_SIZE = 4 * 1024 * 1024;
HEAP_PADDING = 256 * 1024;

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
    var EXPECT_SIZES = 9;
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
    if (data.constructor !== window.ArrayBuffer) {
        memcpy(this.buffer, dest_offset, data.buffer, data.byteOffset, data.byteLength);
    } else {
        memcpy(this.buffer, dest_offset, data, 0, data.byteLength);
    }
};

DynAsm.prototype.initClient = function(gl, assets) {
    // AsmGl must be initialized before calling `client_init`.
    this.asmgl.init(gl);

    var blobs = this._stackAlloc(Int32Array, 7 * 2);
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

DynAsm.prototype.loadTerrainChunk = function(cx, cy, blocks) {
    var buf = this._heapAlloc(Uint16Array, blocks.length);
    buf.set(blocks);
    this._raw['load_terrain_chunk'](this.client,
            cx, cy, buf.byteOffset, buf.byteLength);
    this._heapFree(buf);
};

DynAsm.prototype.prepareGeometry = function(cx0, cy0, cx1, cy1) {
    this._raw['prepare_geometry'](this.client,
            cx0, cy0, cx1, cy1);
};

DynAsm.prototype.getTerrainGeometryBuffer = function() {
    var len_buf = this._stackAlloc(Int32Array, 1);
    var name = this._raw['get_terrain_geometry_buffer'](this.client, len_buf.byteOffset);
    var len = len_buf[0];
    this._stackFree(len_buf);
    return {
        buf: this.asmgl.getBufferWrapper(name),
        len: len / this.SIZEOF.TerrainVertex,
    };
};

DynAsm.prototype.getStructureGeometryBuffer = function() {
    var len_buf = this._stackAlloc(Int32Array, 1);
    var name = this._raw['get_structure_geometry_buffer'](this.client, len_buf.byteOffset);
    var len = len_buf[0];
    this._stackFree(len_buf);
    return {
        buf: this.asmgl.getBufferWrapper(name),
        len: len / this.SIZEOF.StructureVertex,
    };
};

DynAsm.prototype.getLightGeometryBuffer = function() {
    var len_buf = this._stackAlloc(Int32Array, 1);
    var name = this._raw['get_light_geometry_buffer'](this.client, len_buf.byteOffset);
    var len = len_buf[0];
    this._stackFree(len_buf);
    return {
        buf: this.asmgl.getBufferWrapper(name),
        len: len / this.SIZEOF.LightVertex,
    };
};

DynAsm.prototype.getUIGeometryBuffer = function() {
    var len_buf = this._stackAlloc(Int32Array, 1);
    var name = this._raw['get_ui_geometry_buffer'](this.client, len_buf.byteOffset);
    var len = len_buf[0];
    this._stackFree(len_buf);
    return {
        buf: this.asmgl.getBufferWrapper(name),
        len: len / this.SIZEOF.UIVertex,
    };
};


DynAsm.prototype.bench = function() {
    return this._raw['client_bench'](this.client);
};

var module = window['asmlibs_code'];
var static_data = window['asmlibs_data'];

var Vec = require('util/vec').Vec;
var decodeUtf8 = require('util/misc').decodeUtf8;
var LOCAL_SIZE = require('data/chunk').LOCAL_SIZE;


// Memory layout

// Emscripten puts the first static at address 8 to  avoid storing data at
// address 0.
var STATIC_START = 8;  
var STATIC_SIZE = static_data.byteLength;
var STATIC_END = STATIC_START + STATIC_SIZE;

// Align STACK_START to an 8-byte boundary.
var STACK_START = (STATIC_END + 7) & ~7;
// Give at least 16k for stack, and align to 4k boundary.
var STACK_END = (STACK_START + 0x4000 + 0x0fff) & ~0x0fff;
var STACK_SIZE = STACK_END - STACK_START;
console.assert(STACK_SIZE >= 0x1000, 'want at least 4kb for stack');

var HEAP_START = STACK_END;


// External functions

var module_env = function(buffer) {
    var msg_buffer = '';

    return ({
        'abort': function() {
            console.assert(false, 'abort');
            throw 'abort';
        },

        'writeStr': function(ptr, len) {
            var view = new Uint8Array(buffer, ptr, len);
            msg_buffer += decodeUtf8(view);
        },

        'flushStr': function() {
            console.log(msg_buffer);
            msg_buffer = '';
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
    this.buffer = new ArrayBuffer(next_heap_size(INIT_HEAP_SIZE));
    this._memcpy(STATIC_START, static_data);
    this._raw = module(window, module_env(this.buffer), this.buffer);

    this._raw['asmlibs_init']();
    this._raw['asmmalloc_init'](HEAP_START, this.buffer.byteLength);

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
    var EXPECT_SIZES = 11;
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

    result.BlockData = next();
    result.StructureTemplate = next();
    result.TemplatePart = next();
    result.TemplateVertex = next();

    result.BlockChunk = next();

    result.TerrainVertex = next();
    result.StructureVertex = next();
    result.LightVertex = next();

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

DynAsm.prototype.initClient = function(assets) {
    var blobs = this._stackAlloc(Int32Array, 4 * 2);
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
    load_blob(assets['template_defs_bin']);
    load_blob(assets['template_part_defs_bin']);
    load_blob(assets['template_vert_defs_bin']);

    var data = this._stackAlloc(Uint8Array, this.SIZEOF.Data);
    // NB: takes ownership of `blobs`
    this._raw['data_init'](blobs.byteOffset, data.byteOffset);

    this.client = this._raw['asmmalloc_alloc'](this.SIZEOF.Client, this.SIZEOF.ClientAlignment);
    // NB: takes ownership of `data`
    this._raw['client_init'](data.byteOffset, this.client);

    this._stackFree(data);
    this._stackFree(blobs);
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

DynAsm.prototype.structureBufferInsert = function(id, x, y, z, template_id, oneshot_start) {
    return this._raw['structure_buffer_insert'](this.client,
            id, x, y, z, template_id, oneshot_start);
};

DynAsm.prototype.structureBufferRemove = function(idx) {
    return this._raw['structure_buffer_remove'](this.client, idx);
};

DynAsm.prototype.terrainGeomReset = function(cx, cy) {
    this._raw['terrain_geom_reset'](this.client, cx, cy);
};

DynAsm.prototype.terrainGeomGenerate = function() {
    var output = this._stackAlloc(Int32Array, 2);
    var buf = this._heapAlloc(Uint8Array, 256 * 1024);

    this._raw['terrain_geom_generate'](
            this.client,
            buf.byteOffset,
            buf.byteLength,
            output.byteOffset);

    var vertex_count = output[0];
    var more = (output[1] & 1) != 0;
    this._stackFree(output);

    var geom = new Uint8Array(vertex_count * this.SIZEOF.TerrainVertex);
    geom.set(buf.subarray(0, geom.length));
    this._heapFree(buf);

    return {
        geometry: geom,
        more: more,
    };
};

DynAsm.prototype.structureGeomReset = function(cx0, cy0, cx1, cy1, sheet) {
    this._raw['structure_geom_reset'](this.client, cx0, cy0, cx1, cy1, sheet);
};

DynAsm.prototype.structureGeomGenerate = function() {
    var output = this._stackAlloc(Int32Array, 2);
    var buf = this._heapAlloc(Uint8Array, 256 * 1024);

    this._raw['structure_geom_generate'](
            this.client,
            buf.byteOffset,
            buf.byteLength,
            output.byteOffset);

    var vertex_count = output[0];
    var more = (output[1] & 1) != 0;
    this._stackFree(output);

    var geom = new Uint8Array(vertex_count * this.SIZEOF.StructureVertex);
    geom.set(buf.subarray(0, geom.length));
    this._heapFree(buf);

    return {
        geometry: geom,
        more: more,
    };
};

DynAsm.prototype.lightGeomReset = function(cx0, cy0, cx1, cy1) {
    this._raw['light_geom_reset'](this.client, cx0, cy0, cx1, cy1);
};

DynAsm.prototype.lightGeomGenerate = function() {
    var output = this._stackAlloc(Int32Array, 2);
    var buf = this._heapAlloc(Uint8Array, 256 * 1024);

    this._raw['light_geom_generate'](
            this.client,
            buf.byteOffset,
            buf.byteLength,
            output.byteOffset);

    var vertex_count = output[0];
    var more = (output[1] & 1) != 0;
    this._stackFree(output);

    var geom = new Uint8Array(vertex_count * this.SIZEOF.LightVertex);
    geom.set(buf.subarray(0, geom.length));
    this._heapFree(buf);

    return {
        geometry: geom,
        more: more,
    };
};

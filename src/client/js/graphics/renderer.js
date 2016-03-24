var Config = require('config').Config;
var AsmGraphics = require('asmlibs').AsmGraphics;
var SIZEOF = require('asmlibs').SIZEOF;
var getRendererHeapSize = require('asmlibs').getRendererHeapSize;
var getGraphicsHeapSize = require('asmlibs').getGraphicsHeapSize;
var OffscreenContext = require('graphics/canvas').OffscreenContext;
var BlockDef = require('data/chunk').BlockDef;
var TemplateDef = require('data/templates').TemplateDef;
var CHUNK_SIZE = require('data/chunk').CHUNK_SIZE;
var TILE_SIZE = require('data/chunk').TILE_SIZE;
var LOCAL_SIZE = require('data/chunk').LOCAL_SIZE;
var Texture = require('graphics/glutil').Texture;
var Buffer = require('graphics/glutil').Buffer;
var Framebuffer = require('graphics/glutil').Framebuffer;
var makeShaders = require('graphics/shaders').makeShaders;
var BufferCache = require('graphics/buffers').BufferCache;
var CavernMap = require('graphics/cavernmap').CavernMap;

var GlObject = require('graphics/glutil').GlObject;
var uniform = require('graphics/glutil').uniform;
var attribute = require('graphics/glutil').attribute;

var PonyAppearanceClass = require('graphics/appearance/pony').PonyAppearanceClass;

var TimeSeries = require('util/timeseries').TimeSeries;
var fstr1 = require('util/misc').fstr1;


var CHUNK_PX = CHUNK_SIZE * TILE_SIZE;

// The `now` value passed to the animation shader must be reduced to fit in a
// float.  We use the magic number 55440 for this, since it's divisible by
// every number from 1 to 12 (and most "reasonable" numbers above that).  This
// is useful because repeating animations will glitch when `now` wraps around
// unless `length / framerate` divides evenly into the modulus.
//
// Note that the shader `now` and ANIM_MODULUS are both in seconds, not ms.
var ANIM_MODULUS = 55440;

// We also need a smaller modulus for one-shot animation start times.  These
// are measured in milliseconds and must fit in a 16-bit int.  It's important
// that the one-shot modulus divides evenly into 1000 * ANIM_MODULUS, because
// the current frame time in milliseconds will be modded by 1000 * ANIM_MODULUS
// and then again by the one-shot modulus.
//
// We re-use ANIM_MODULUS as the one-shot modulus, since it obviously divides
// evenly into 1000 * ANIM_MODULUS.  This is okay as long as ANIM_MODULUS fits
// into 16 bits.
var ONESHOT_MODULUS = ANIM_MODULUS;


/** @constructor */
function RenderData(gl, asm) {
    this.gl = gl;
    this._asm = asm;

    this.texture_cache = new WeakMap();

    var r = this;

    // TODO: move these somewhere nicer

    this.terrain_stale = false;

    this.structure_buf = new BufferCache(gl, function(cx, cy, feed) {
        r._asm.structureGeomReset(cx, cy, cx + 1, cy + 1);
        var more = true;
        while (more) {
            var result = r._asm.structureGeomGenerate();
            feed(result.geometry);
            more = result.more;
        }
    });

    this.light_buf = new BufferCache(gl, function(cx, cy, feed) {
        r._asm.lightGeomReset(cx, cy, cx + 1, cy + 1);
        var more = true;
        while (more) {
            var result = r._asm.lightGeomGenerate();
            feed(result.geometry);
            more = result.more;
        }
    });

    this.cavern_map = new CavernMap(gl, 32);
}

RenderData.prototype.prepare = function(scene) {
    var pos = scene.camera_pos;
    var size = scene.camera_size;

    var cx0 = ((pos[0]|0) / CHUNK_PX)|0;
    var cx1 = (((pos[0]|0) + (size[0]|0) + CHUNK_PX) / CHUNK_PX)|0;
    var cy0 = ((pos[1]|0) / CHUNK_PX)|0;
    var cy1 = (((pos[1]|0) + (size[1]|0) + CHUNK_PX) / CHUNK_PX)|0;

    // Terrain from the chunk below can cover the current one.
    if (this.terrain_stale) {
        this._asm.updateTerrainGeometry(cx0, cy0, cx1, cy1 + 1);
        this.terrain_stale = false;
    }
    // Structures from the chunk below can cover the current one, and also
    // structures from chunks above and to the left can extend into it.
    this.structure_buf.prepare(cx0 - 1, cy0 - 1, cx1, cy1 + 1);
    // Light from any adjacent chunk can extend into the current one.
    this.light_buf.prepare(cx0 - 1, cy0 - 1, cx1 + 1, cy1 + 1);
};

// RenderData initialization

RenderData.prototype.initData = function(blocks, templates, parts, verts) {
    this._asm = new AsmGraphics(
            blocks.length, templates.length, parts.length, verts.length / 3,
            512 * 1024, 512 * 1024);

    this._asm.terrainGeomInit();
    this._asm.structureBufferInit();
    this._asm.structureGeomInit();
    this._asm.lightGeomInit();

    this.loadBlockData(blocks);
    this.loadTemplateData(templates);
    this.loadTemplateParts(parts);
    this.loadTemplateVerts(verts);
};

// Texture object management

RenderData.prototype.cacheTexture = function(image) {
    var tex = this.texture_cache.get(image);
    if (tex != null) {
        // Cache hit
        return tex;
    }

    // Cache miss - create a new texture
    var tex = new Texture(this.gl);
    tex.loadImage(image);
    this.texture_cache.set(image, tex);
    return tex;
};

RenderData.prototype.refreshTexture = function(image) {
    var tex = this.texture_cache.get(image);
    if (tex != null) {
        tex.loadImage(image);
    }
};


// Data loading

// Helper function for writing data into an asm structure.  Constructs a
// subarray of `view` for accessing element `index` in an array of structures
// of the given `size`.  The `size` should include any necessary padding for
// alignment following each structure.
function mk_out(view, index, size) {
    var shift;
    switch (view.constructor.BYTES_PER_ELEMENT) {
        case 1: shift = 0; break;
        case 2: shift = 1; break;
        case 4: shift = 2; break;
        case 8: shift = 3; break;
        default: throw 'TypedArray has non-power-of-two BYTES_PER_ELEMENT';
    }
    var arr = view.subarray(index * (size >> shift), (index + 1) * (size >> shift));

    // If `count` is null, store number `x` at byte offset `j`.  Otherwise,
    // store `count` numbers from array `x` starting at byte offset `j`.
    return function(j, x, count) {
        if (count == null) {
            arr[j >> shift] = x;
        } else {
            for (var k = 0; k < count; ++k) {
                arr[(j >> shift) + k] = x[k];
            }
        }
    };
}

RenderData.prototype.loadBlockData = function(blocks) {
    var view8 = this._asm.blockDataView8();
    var view16 = this._asm.blockDataView16();
    for (var i = 0; i < blocks.length; ++i) {
        var block = blocks[i];
        var out8 = mk_out(view8, i, SIZEOF.BlockData);
        var out16 = mk_out(view16, i, SIZEOF.BlockData);

        out16(  0, block.front);
        out16(  2, block.back);
        out16(  4, block.top);
        out16(  6, block.bottom);

        out8(   8, block.light_color, 3);
        out8(  11, block.shape);
        out16( 12, block.light_radius);
    }
};

RenderData.prototype.loadChunk = function(i, j, chunk) {
    this._asm.loadTerrainChunk(j, i, chunk._tiles);

    this.terrain_stale = true;
    this.cavern_map.invalidate();
};

RenderData.prototype.loadTemplateData = function(templates) {
    var view8 = this._asm.templateDataView8();
    var view16 = this._asm.templateDataView16();

    for (var i = 0; i < templates.length; ++i) {
        var template = templates[i];
        var out8 = mk_out(view8, i, SIZEOF.StructureTemplate);
        var out16 = mk_out(view16, i, SIZEOF.StructureTemplate);

        out8(   0, template.size.x);
        out8(   1, template.size.y);
        out8(   2, template.size.z);
        out16(  4, template.part_idx);
        out8(   6, template.part_count);
        out8(   7, template.vert_count);
        out8(   8, template.layer);
        out8(   9, template.flags);

        out8(  10, template.light_pos, 3);
        out8(  13, template.light_color, 3);
        out16( 16, template.light_radius);
    }
};

RenderData.prototype.loadTemplateParts = function(parts) {
    var view8 = this._asm.templatePartView8();
    var view16 = this._asm.templatePartView16();

    for (var i = 0; i < parts.length; ++i) {
        var part = parts[i];
        var out8 = mk_out(view8, i, SIZEOF.TemplatePart);
        var out16 = mk_out(view16, i, SIZEOF.TemplatePart);

        out16(  0, part.vert_idx);
        out16(  2, part.vert_count);
        out16(  4, part.offset[0]);
        out16(  6, part.offset[1]);
        out8(   8, part.sheet);
        out8(   9, part.flags);

        var oneshot_length = part.anim_length * (part.anim_oneshot ? -1 : 1);
        out8(  10, oneshot_length);
        out8(  11, part.anim_rate);
        out16( 12, part.anim_size[0]);
    }
};

RenderData.prototype.loadTemplateVerts = function(verts) {
    console.assert(SIZEOF.TemplateVertex == 6);
    var view = this._asm.templateVertexView();
    view.set(verts);
};

RenderData.prototype.addStructure = function(now, id, x, y, z, template) {
    var tx = (x / TILE_SIZE) & (LOCAL_SIZE * CHUNK_SIZE - 1);
    var ty = (y / TILE_SIZE) & (LOCAL_SIZE * CHUNK_SIZE - 1);
    var tz = (z / TILE_SIZE) & (LOCAL_SIZE * CHUNK_SIZE - 1);

    var oneshot_start = now % ONESHOT_MODULUS;
    if (oneshot_start < 0) {
        oneshot_start += ONESHOT_MODULUS;
    }
    var render_idx = this._asm.structureAppear(
            id, tx, ty, tz, template.id, oneshot_start);

    this._invalidateStructure(tx, ty, tz, template);
    return render_idx;
};

RenderData.prototype.removeStructure = function(structure) {
    this._asm.structureGone(structure.render_index);

    var pos = structure.pos;
    this._invalidateStructure(pos.x, pos.y, pos.z, structure.template);
};

RenderData.prototype._invalidateStructure = function(x, y, z, template) {
    var cx = (x / CHUNK_SIZE)|0;
    var cy = (y / CHUNK_SIZE)|0;

    this.structure_buf.invalidate(cx, cy);
    // TODO: magic number
    if (template.flags & 4) {   // HAS_LIGHT
        this.light_buf.invalidate(cx, cy);
    }
    this.cavern_map.invalidate();
};


// RenderBuffers

/** @constructor */
function RenderBuffers(gl) {
    this.gl = gl;

    // Temporary buffers
    this.fb_world = null;
    this.fb_light = null;
    this.fb_shadow = null;

    // Output buffers for specific layers
    this.fb_layer0 = null;
    this.fb_layer1 = null;

    this.fb_final = null;

    this.last_sw = -1;
    this.last_sh = -1;
}

RenderBuffers.prototype.prepare = function(scene) {
    var sw = scene.camera_size[0];
    var sh = scene.camera_size[1];
    if (sw == this.last_sw && sh == this.last_sh) {
        return;
    }

    // Framebuffer containing image and metadata for the world (terrain +
    // structures).
    this.fb_world = new Framebuffer(this.gl, sw, sh, 2);
    // Framebuffer containing light intensity at every pixel.
    this.fb_light = new Framebuffer(this.gl, sw, sh, 1, false);
    // Temporary framebuffer for storing shadows and other translucent parts
    // during structure rendering.
    this.fb_shadow = new Framebuffer(this.gl, sw, sh, 1);

    // Output framebuffers for fully-rendered scenes.  We use more than one
    // because each may have different slicing applied.
    this.fb_layer0 = new Framebuffer(this.gl, sw, sh, 1, false);
    this.fb_layer1 = new Framebuffer(this.gl, sw, sh, 1, false);

    // Framebuffer containing the final image data.  This is emitted directly
    // to the screen.  (May require upscaling, which is why the postprocessing
    // shader doesn't output to the screen immediately.)
    this.fb_final = new Framebuffer(this.gl, sw, sh, 1, false);

    this.last_sw = sw;
    this.last_sh = sh;
};



// RenderShaders

/** @constructor */
function RenderShaders(gl, assets, data, shader_defs) {
    this.gl = gl;
    makeShaders(this, gl, assets, shader_defs,
            function(img) { return data.cacheTexture(img) });

    this.class_list = [new PonyAppearanceClass(gl, assets, shader_defs)];
    this.classes = new WeakMap();
    for (var i = 0; i < this.class_list.length; ++i) {
        var cls = this.class_list[i];
        this.classes.set(cls.constructor, cls);
    }
}

RenderShaders.prototype.prepare = function(scene) {
    var pos = scene.camera_pos;
    var size = scene.camera_size;
    var slice_z = [scene.slice_z];
    var slice_center = scene.slice_center;

    var anim_now_val = scene.now / 1000 % ANIM_MODULUS;
    if (anim_now_val < 0) {
        anim_now_val += ANIM_MODULUS;
    }
    var anim_now = [anim_now_val];

    this.terrain.setUniformValue('cameraPos', pos);
    this.terrain.setUniformValue('cameraSize', size);
    this.terrain.setUniformValue('sliceCenter', slice_center);
    this.terrain.setUniformValue('sliceZ', slice_z);
    this.structure.setUniformValue('cameraPos', pos);
    this.structure.setUniformValue('cameraSize', size);
    this.structure.setUniformValue('sliceCenter', slice_center);
    this.structure.setUniformValue('sliceZ', slice_z);
    this.structure.setUniformValue('now', anim_now);
    this.structure_shadow.setUniformValue('cameraPos', pos);
    this.structure_shadow.setUniformValue('cameraSize', size);
    this.structure_shadow.setUniformValue('sliceCenter', slice_center);
    this.structure_shadow.setUniformValue('sliceZ', slice_z);
    this.structure_shadow.setUniformValue('now', anim_now);
    this.light_static.setUniformValue('cameraPos', pos);
    this.light_static.setUniformValue('cameraSize', size);
    this.light_dynamic.setUniformValue('cameraPos', pos);
    this.light_dynamic.setUniformValue('cameraSize', size);
    // this.blit_full uses fixed camera

    if (this.blend_layers) {
        this.blend_layers.setUniformValue('cameraPos', pos);
        this.blend_layers.setUniformValue('cameraSize', size);
        this.blend_layers.setUniformValue('sliceCenter', slice_center);
        this.blend_layers.setUniformValue('sliceZ', slice_z);
    }

    for (var i = 0; i < this.class_list.length; ++i) {
        var cls = this.class_list[i];
        cls.setCamera(pos, size, slice_center, slice_z);
    }
}

RenderShaders.prototype.renderLayer = function(scene, data, buffers, out_buf) {
    var gl = this.gl;

    var size = scene.camera_size;

    // Render everything into the world framebuffer.

    gl.viewport(0, 0, size[0], size[1]);
    gl.clearDepth(0.0);
    gl.clearColor(0, 0, 0, 0);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.GEQUAL);

    var this_ = this;

    buffers.fb_world.use(function(fb_idx) {
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

        var terrain_info = data._asm.getTerrainGeometryBuffer();
        var buf = terrain_info.buf;
        var len = terrain_info.len;
        this_.terrain.draw(fb_idx, 0, len / SIZEOF.TerrainVertex, {}, {'*': buf}, {
            'cavernTex': data.cavern_map.getTexture(),
        });

        var buf = data.structure_buf.getBuffer();
        var len = data.structure_buf.getSize();
        this_.structure.draw(fb_idx, 0, len / SIZEOF.StructureVertex, {}, {'*': buf}, {
            'cavernTex': data.cavern_map.getTexture(),
        });

        for (var i = 0; i < scene.sprites.length; ++i) {
            var sprite = scene.sprites[i];
            var cls = this_.classes.get(sprite.appearance.getClass());
            cls.draw3D(fb_idx, data, sprite);
        }
    });

    buffers.fb_shadow.use(function(fb_idx) {
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

        var buf = data.structure_buf.getBuffer();
        var len = data.structure_buf.getSize();
        this_.structure_shadow.draw(fb_idx, 0, len / SIZEOF.StructureVertex, {}, {'*': buf}, {
            'cavernTex': data.cavern_map.getTexture(),
        });
    });

    gl.disable(gl.DEPTH_TEST);


    // Render lights into the light framebuffer.

    gl.enable(gl.BLEND);
    gl.blendFunc(gl.ONE, gl.ONE);
    // clearColor sets the ambient light color+intensity
    var amb = scene.ambient_color;
    var amb_intensity = 0.2126 * amb[0] + 0.7152 * amb[1] + 0.0722 * amb[2];
    gl.clearColor(amb[0] / 255, amb[1] / 255, amb[2] / 255, amb_intensity / 255);

    buffers.fb_light.use(function(fb_idx) {
        gl.clear(gl.COLOR_BUFFER_BIT);

        var buf = data.light_buf.getBuffer();
        var len = data.light_buf.getSize();
        this_.light_static.draw(fb_idx, 0, len / SIZEOF.LightVertex, {}, {'*': buf}, {
            'depthTex': buffers.fb_world.depth_texture,
        });

        for (var i = 0; i < scene.lights.length; ++i) {
            var light = scene.lights[i];
            this_.light_dynamic.draw(fb_idx, 0, 6, {
                'center': [
                    light.pos.x,
                    light.pos.y,
                    light.pos.z,
                ],
                'colorIn': [
                    light.color[0] / 255,
                    light.color[1] / 255,
                    light.color[2] / 255,
                ],
                'radiusIn': [light.radius],
            }, {}, {
                'depthTex': buffers.fb_world.depth_texture,
            });
        }
    });

    gl.disable(gl.BLEND);


    // Apply post-processing pass

    out_buf.use(function(idx) {
        this_.post_filter.draw(idx, 0, 6, {
            'screenSize': size,
        }, {}, {
            'image0Tex': buffers.fb_world.textures[0],
            'image1Tex': buffers.fb_world.textures[1],
            'lightTex': buffers.fb_light.textures[0],
            'depthTex': buffers.fb_world.depth_texture,
            'shadowTex': buffers.fb_shadow.textures[0],
            'shadowDepthTex': buffers.fb_shadow.depth_texture,
        });
    });
};


/** @constructor */
function Renderer(gl, assets, asm) {
    this.gl = gl;
    this.data = new RenderData(gl, asm);
    this.buffers = new RenderBuffers(gl);
    this.simple_slice = Config.render_simplified_slicing.get();
    if (!this.simple_slice) {
        this.shaders = new RenderShaders(gl, assets, this.data);
        this.shaders_slice = new RenderShaders(gl, assets, this.data,
                {'SLICE_ENABLE': '1'});
    } else {
        this.shaders = new RenderShaders(gl, assets, this.data,
                {'SLICE_ENABLE': '1', 'SLICE_SIMPLIFIED': '1'});
    }

    this.prep_time = new TimeSeries(5000);
    this.render_time = new TimeSeries(5000);
}
exports.Renderer = Renderer;

Renderer.prototype.initData = function(blocks, templates, parts, verts) {
    this.data.initData(blocks, templates, parts, verts);
};

Renderer.prototype.cacheTexture = function(img) {
    this.data.cacheTexture(img);
};

Renderer.prototype.refreshTexture = function(img) {
    this.data.refreshTexture(img);
};

Renderer.prototype.loadChunk = function(i, j, chunk) {
    this.data.loadChunk(i, j, chunk);
};

Renderer.prototype.addStructure = function(now, id, x, y, z, template) {
    return this.data.addStructure(now, id, x, y, z, template);
};

Renderer.prototype.removeStructure = function(structure) {
    return this.data.removeStructure(structure);
};

Renderer.prototype.updateCavernMap = function(phys_asm, pos) {
    if (this.data.cavern_map.needsUpdate(pos)) {
        this.data.cavern_map.update(phys_asm, pos);
    }
};

Renderer.prototype.render = function(scene, draw_ui) {
    // Prepare all components for rendering
    var start_prep = Date.now();

    this.data.prepare(scene);
    this.buffers.prepare(scene);
    this.shaders.prepare(scene);
    if (!this.simple_slice) {
        this.shaders_slice.prepare(scene);
    }

    var end_prep = Date.now();


    // Render
    var start_render = end_prep;

    var gl = this.gl;
    var this_ = this;

    if (!this.simple_slice) {
        this.shaders.renderLayer(scene, this.data, this.buffers, this.buffers.fb_layer0);
        this.shaders_slice.renderLayer(scene, this.data, this.buffers, this.buffers.fb_layer1);

        // Copy output framebuffer to canvas.

        this.buffers.fb_final.use(function(fb_idx) {
            this_.shaders.blend_layers.draw(fb_idx, 0, 6, {}, {}, {
                'baseTex': this_.buffers.fb_layer0.textures[0],
                'slicedTex': this_.buffers.fb_layer1.textures[0],
                'cavernTex': this_.data.cavern_map.getTexture(),
            });
        });
    } else {
        this.shaders.renderLayer(scene, this.data, this.buffers, this.buffers.fb_final);
    }

    if (draw_ui != null && !window.hideUI) {
        draw_ui(scene.camera_size, this.buffers.fb_final);
    }

    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    // TODO: move blit_full to a common location
    this.shaders.blit_full.draw(0, 0, 6, {}, {}, {
        'imageTex': this.buffers.fb_final.textures[0],
    });

    var end_render = Date.now();
    this.prep_time.record(end_prep, end_prep - start_prep);
    this.render_time.record(end_render, end_render - start_render);
};

Renderer.prototype.getDebugHTML = function() {
    var prep_sum = this.prep_time.sum;
    var prep_ms = this.prep_time.sum / this.prep_time.count;
    var render_ms = this.render_time.sum / this.render_time.count;
    return (
        'Prep: ' + fstr1(prep_ms) + ' ms<br>' +
        'Prep (sum): ' + prep_sum + ' ms<br>' +
        'Render: ' + fstr1(render_ms) + ' ms'
        );
};

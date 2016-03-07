var ItemDef = require('data/items').ItemDef;
var G = require('graphics/glutil');
var SB = require('graphics/shaderbuilder');
var TILE_SIZE = require('data/chunk').TILE_SIZE;

/** @constructor */
function UIRenderContext(gl, assets) {
    this.gl = gl;
    this.assets = assets;
    this.shaders = makeUIShaders(gl, assets);
    this.textures = makeUITextures(gl, assets);
    this.buffers = new UIRenderBuffers(assets['ui_atlas_parts']);
}
exports.UIRenderContext = UIRenderContext;

function makeTexture(gl, img) {
    var tex = new G.Texture(gl);
    tex.loadImage(img);
    return tex;
}

function makeUITextures(gl, assets) {
    return {
        ui_atlas: makeTexture(gl, assets['ui_atlas']),
        items: makeTexture(gl, assets['items_img']),
        font: makeTexture(gl, assets['fonts']),
    };
}

function makeUIShaders(gl, assets) {
    var s = {};
    var ctx = new SB.ShaderBuilderContext(gl, assets, {},
            function(img) { return makeTexture(gl, img); });

    s.blit = ctx.start('ui_blit.vert', 'ui_blit.frag')
        .uniformVec2('screenSize')
        .uniformVec2('sheetSize')
        .attributes(new SB.Attributes(8)
                .field(0, gl.SHORT, 2, 'source')
                .field(4, gl.SHORT, 2, 'dest'))
        .texture('sheet', null)
        .finish();

    s.blit_tiled = ctx.start('ui_blit_tiled.vert', 'ui_blit_tiled.frag')
        .uniformVec2('screenSize')
        .uniformVec2('sheetSize')
        .attributes(new SB.Attributes(16)
                .field(0, gl.UNSIGNED_SHORT, 2, 'srcPos')
                .field(4, gl.UNSIGNED_SHORT, 2, 'srcSize')
                .field(8, gl.SHORT, 2, 'srcStepPx')
                .field(12, gl.SHORT, 2, 'dest'))
        .texture('sheet', null)
        .finish();

    return s;
}

UIRenderContext.prototype.updateBuffers = function(root) {
    if (root._layoutDamaged) {
        root.runLayout();
        this._walkUpdateLayout(root);
    }
    if (root._damaged) {
        this.buffers.reset();
        this._walkUpdateBuffers(root, 0, 0);
    }
};

UIRenderContext.prototype._walkUpdateLayout = function(w) {
    // Don't need to runLayout() for each widget since the root runLayout()
    // operates recursively.
    w._layoutDamaged = false;
    w.damage();
};

UIRenderContext.prototype._walkUpdateBuffers = function(w, x, y) {
    w.render(this.buffers, x, y);
    w._damaged = false;
    for (var i = 0; i < w.children.length; ++i) {
        var c = w.children[i];
        this._walkUpdateBuffers(c, x + c._x, y + c._y);
    }
};

UIRenderContext.prototype.render = function(root, size, fb) {
    this.updateBuffers(root);

    var gl = this.gl;
    gl.viewport(0, 0, size[0], size[1]);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.GEQUAL);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    var this_ = this;
    fb.use(function(fb_idx) {
        this_.shaders.blit_tiled.draw(fb_idx,
                0, this_.buffers.ui_atlas.length / 8,
                {
                    'screenSize': size,
                    'sheetSize': [256, 256],
                },
                { '*': this_.buffers.ui_atlas.getGlBuffer(gl) },
                { 'sheet': this_.textures.ui_atlas });

        this_.shaders.blit.draw(fb_idx,
                0, this_.buffers.items.length / 4,
                {
                    'screenSize': size,
                    'sheetSize': [1024, 1024],
                },
                { '*': this_.buffers.items.getGlBuffer(gl) },
                { 'sheet': this_.textures.items });

        var font_tex = this_.textures.font;
        this_.shaders.blit.draw(fb_idx,
                0, this_.buffers.text.length / 4,
                {
                    'screenSize': size,
                    'sheetSize': [font_tex.width, font_tex.height],
                },
                { '*': this_.buffers.text.getGlBuffer(gl) },
                { 'sheet': font_tex });
    });

    gl.disable(gl.DEPTH_TEST);
};


/** @constructor */
function UIRenderBuffers(ui_parts) {
    this.ui_atlas = new UIBuffer();
    this.items = new UIBuffer();
    this.text = new UIBuffer();

    this.ui_parts = ui_parts;
}

UIRenderBuffers.prototype.reset = function() {
    this.ui_atlas = new UIBuffer();
    this.items = new UIBuffer();
    this.text = new UIBuffer();
};

function addQuad(buf, sx, sy, sw, sh, dx, dy, dw, dh) {
    buf.push(sx + 0,  sy + 0,  dx + 0,  dy + 0);
    buf.push(sx + 0,  sy + sh, dx + 0,  dy + dh);
    buf.push(sx + sw, sy + 0,  dx + dw, dy + 0);

    buf.push(sx + sw, sy + 0,  dx + dw, dy + 0);
    buf.push(sx + 0,  sy + sh, dx + 0,  dy + dh);
    buf.push(sx + sw, sy + sh, dx + dw, dy + dh);
}

UIRenderBuffers.prototype.drawUI = function(key, dx, dy, dw, dh) {
    var part = this.ui_parts[key];
    var sx = part['x'];
    var sy = part['y'];
    var sw = part['w'];
    var sh = part['h'];

    if (dw == null) {
        dw = sw;
    }
    if (dh == null) {
        dh = sh;
    }

    // Provide additional vertex data to allow for tiling.
    this.ui_atlas.push(sx, sy, sw, sh,  0,  0,  dx + 0,  dy + 0);
    this.ui_atlas.push(sx, sy, sw, sh,  0,  dh, dx + 0,  dy + dh);
    this.ui_atlas.push(sx, sy, sw, sh,  dw, 0,  dx + dw, dy + 0);

    this.ui_atlas.push(sx, sy, sw, sh,  dw, 0,  dx + dw, dy + 0);
    this.ui_atlas.push(sx, sy, sw, sh,  0,  dh, dx + 0,  dy + dh);
    this.ui_atlas.push(sx, sy, sw, sh,  dw, dh, dx + dw, dy + dh);
};

UIRenderBuffers.prototype.drawItem = function(id, dx, dy) {
    var def = ItemDef.by_id[id];
    addQuad(this.items,
            def.tile_x * TILE_SIZE, def.tile_y * TILE_SIZE,
            TILE_SIZE, TILE_SIZE,
            dx, dy,
            16, 16);
};

UIRenderBuffers.prototype.drawChar = function(sx, sy, w, h, dx, dy) {
    addQuad(this.text,
            sx, sy, w, h,
            dx, dy, w, h);
};


/** @constructor */
function UIBuffer() {
    this.data = new Int16Array(16);
    this.length = 0;
    this.gl_data = null;
}

UIBuffer.prototype.push = function() {
    while (this.length + arguments.length >= this.data.length) {
        this._grow();
    }

    for (var i = 0; i < arguments.length; ++i) {
        this.data[this.length] = arguments[i];
        ++this.length;
    }
};

UIBuffer.prototype._grow = function() {
    var new_data = new Int16Array(this.data.length * 2);
    new_data.set(this.data);
    this.data = new_data;
};

UIBuffer.prototype.getGlBuffer = function(gl) {
    if (this.gl_data == null) {
        this.gl_data = new G.Buffer(gl);
        this.gl_data.loadData(this.data.subarray(0, this.length));
    }
    return this.gl_data;
}

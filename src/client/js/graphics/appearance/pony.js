var Config = require('config').Config;
var Sprite = require('graphics/sprite').Sprite;
var OffscreenContext = require('graphics/canvas').OffscreenContext;
var glutil = require('graphics/glutil');
var named = require('graphics/draw/named');
var sb = require('graphics/shaderbuilder');

var AttachSlotDef = require('data/attachments').AttachSlotDef;
var ExtraDefs = require('data/extras').ExtraDefs;


var TRIBE_NAME = ['E', 'P', 'U', 'A'];
var COLOR_RAMP = [0x44, 0x88, 0xcc, 0xff];


/** @constructor */
function PonyAppearanceClass(gl, assets) {
    this._name_tex = new glutil.Texture(gl);
    this._name_buf = new named.NameBuffer(assets);

    var shaders = makePonyShaders(gl, assets, this._name_tex);
    this._obj = shaders.pony;
    this._name_obj = shaders.name;
}
exports.PonyAppearanceClass = PonyAppearanceClass;

PonyAppearanceClass.prototype.setCamera = function(pos, size, slice_center, slice_z) {
    this._obj.setUniformValue('cameraPos', pos);
    this._obj.setUniformValue('cameraSize', size);
    this._obj.setUniformValue('sliceCenter', slice_center);
    this._obj.setUniformValue('sliceZ', slice_z);

    this._name_obj.setUniformValue('cameraPos', pos);
    this._name_obj.setUniformValue('cameraSize', size);
    this._name_obj.setUniformValue('sliceCenter', slice_center);
    this._name_obj.setUniformValue('sliceZ', slice_z);
};

PonyAppearanceClass.prototype.getNameOffset = function(name) {
    var off = this._name_buf.offset(name);
    if (off.created) {
        this._name_tex.loadImage(this._name_buf.image());
    }
    return off;
};

function makePonyShaders(gl, assets, name_tex) {
    var ctx = new sb.ShaderBuilderContext(gl, assets, null);
    var shaders = {};

    var square_buf = ctx.makeBuffer(new Uint8Array([
        0, 0,
        0, 1,
        1, 1,

        0, 0,
        1, 1,
        1, 0,
    ]));

    var sprite_uniforms = new sb.Uniforms()
        .vec2('cameraPos')
        .vec2('cameraSize')
        .vec2('sheetSize')
        .vec2('sliceCenter')
        .float_('sliceZ')
        .vec3('pos')
        .vec2('base')
        .vec2('size')
        .vec2('anchor');

    var sprite_attributes = new sb.Attributes(2, square_buf)
        .field( 0, gl.UNSIGNED_BYTE, 2, 'posOffset');

    shaders.pony = ctx.start('sprite.vert', 'app_pony.frag', 2)
        .uniforms(sprite_uniforms)
        .uniformVec3('colorBody')
        .uniformVec3('colorHair')
        .uniformBool('hasEquip')
        .attributes(sprite_attributes)
        .texture('sheetBase')
        .texture('sheetMane')
        .texture('sheetTail')
        .texture('sheetEyes')
        .texture('sheetEquip[0]')
        .texture('sheetEquip[1]')
        .texture('sheetEquip[2]')
        .texture('cavernTex')
        .finish();

    shaders.name = ctx.start('sprite.vert', 'sprite.frag', 2)
        .uniforms(sprite_uniforms)
        .attributes(sprite_attributes)
        .texture('sheetSampler', name_tex)
        .texture('cavernTex')
        .finish();
    shaders.name.setUniformValue('sheetSize',
            [named.NAME_BUFFER_WIDTH, named.NAME_BUFFER_HEIGHT]);

    return shaders;
}

PonyAppearanceClass.prototype.draw3D = function(fb_idx, data, sprite) {
    var app = sprite.appearance;

    var base_tex = data.cacheTexture(app.base_img);
    var textures = {
        'sheetBase': base_tex,
        'sheetEyes': data.cacheTexture(app.eyes_img),
        'sheetMane': data.cacheTexture(app.mane_img),
        'sheetTail': data.cacheTexture(app.tail_img),
        'cavernTex': data.cavern_map.getTexture(),
    };

    for (var i = 0; i < 3; ++i) {
        if (app.equip_img[i] != null) {
            textures['sheetEquip[' + i + ']'] = data.cacheTexture(app.equip_img[i]);
        }
    }

    var offset_x = sprite.frame_j * sprite.width;
    var offset_y = sprite.frame_i * sprite.height;

    var uniforms = {
        'sheetSize': [base_tex.width, base_tex.height],
        'pos': [sprite.ref_x, sprite.ref_y, sprite.ref_z],
        'base': [offset_x + (sprite.flip ? sprite.width : 0),
                 offset_y],
        'size': [(sprite.flip ? -sprite.width : sprite.width),
                 sprite.height],
        'anchor': [sprite.anchor_x, sprite.anchor_y],

        'colorBody': app.body_color,
        'colorHair': app.hair_color,
        'hasEquip': app.has_equip,
    };

    this._obj.draw(fb_idx, 0, 6, uniforms, {}, textures);


    if (Config.render_names.get()) {
        var off = this.getNameOffset(app.name);

        var uniforms = {
            // TODO: hardcoded name positioning, should be computed somehow to
            // center the name at a reasonable height.
            'pos': [sprite.ref_x, sprite.ref_y, sprite.ref_z + 90 - 22],
            'base': [off.x, off.y],
            'size': [named.NAME_WIDTH, named.NAME_HEIGHT],
            'anchor': [named.NAME_WIDTH / 2, named.NAME_HEIGHT],
        };
        this._name_obj.draw(fb_idx, 0, 6, uniforms, {}, {
            'cavernTex': data.cavern_map.getTexture(),
        });
    }
};

PonyAppearanceClass.prototype.draw2D = function(ctx, view_base, sprite) {
    var app = sprite.appearance;

    var x = sprite.ref_x - sprite.anchor_x - view_base[0];
    var y = sprite.ref_y - sprite.ref_z - sprite.anchor_y - view_base[1];
    var w = sprite.width;
    var h = sprite.height;

    var buf = new OffscreenContext(w, h);
    var buf_x = 0;
    var buf_y = 0;

    if (sprite.flip) {
        buf.scale(-1, 1);
        buf_x = -buf_x - w;
    }

    var off_x = sprite.frame_j * w;
    var off_y = sprite.frame_i * h;

    // TODO: fix alpha
    function draw_layer(img) {
        buf.globalCompositeOperation = 'copy';
        buf.drawImage(img,
                off_x, off_y, w, h,
                buf_x, buf_y, w, h);
        var img = buf.getImageData(0, 0, w, h);

        for (var i = 3; i < img.data.length; i += 4) {
            if (img.data[i] != 0) {
                img.data[i] = 255;
            }
        }
        buf.putImageData(img, 0, 0);

        ctx.drawImage(buf.canvas, x, y);
    }

    function draw_layer_tinted(img, color) {
        buf.globalCompositeOperation = 'copy';
        buf.drawImage(img,
                off_x, off_y, w, h,
                buf_x, buf_y, w, h);
        var orig = buf.getImageData(0, 0, w, h);

        buf.globalCompositeOperation = 'multiply';
        buf.fillStyle = 'rgb(' + [color[0] * 255, color[1] * 255, color[2] * 255].join(',') + ')';
        buf.fillRect(buf_x, buf_y, w, h);
        var img = buf.getImageData(0, 0, w, h);

        for (var i = 3; i < img.data.length; i += 4) {
            if (orig.data[i] == 0) {
                img.data[i] = 0;
            } else {
                img.data[i] = 255;
            }
        }
        buf.putImageData(img, 0, 0);

        ctx.drawImage(buf.canvas, x, y);
    }

    draw_layer_tinted(app.base_img, app.body_color);
    draw_layer(app.eyes_img);
    draw_layer_tinted(app.mane_img, app.hair_color);
    draw_layer_tinted(app.tail_img, app.hair_color);
};


/** @constructor */
function PonyAppearance(assets, bits, name) {
    var stallion = (bits >> 8) & 1;
    var base = (bits >> 6) & 3;
    var mane = (bits >> 10) & 7;
    var tail = (bits >> 13) & 7;
    var eyes = (bits >> 16) & 3;
    var equip0 = (bits >> 18) & 15;
    var equip1 = (bits >> 22) & 15;
    var equip2 = (bits >> 26) & 15;

    // TODO: use a SpriteSheet object that contains all the sheet images
    var base_idx = ExtraDefs.pony_bases_table[base];

    function get_image(slot_key, attachment_id, sheet_index) {
        var slot_idx = ExtraDefs.pony_slot_table[stallion][slot_key];
        var slot = AttachSlotDef.by_id[slot_idx];
        var file_name = slot.sprite_files[attachment_id];
        return file_name ? assets[file_name + '-' + sheet_index] : null;
    }

    this.base_img = get_image('base', base_idx, 0);
    this.eyes_img = get_image('eyes', eyes, 0);
    this.mane_img = get_image('mane', mane, 0);
    this.tail_img = get_image('tail', tail, 0);

    this.equip_img = [
        get_image('equip0', equip0, 0),
        get_image('equip1', equip1, 0),
        get_image('equip2', equip2, 0),
    ];
    this.has_equip = [
        equip0 != 0,
        equip1 != 0,
        equip2 != 0,
    ];

    var r = (bits >> 4) & 3;
    var g = (bits >> 2) & 3;
    var b = (bits >> 0) & 3;
    this.hair_color = [
        COLOR_RAMP[r] / 255.0,
        COLOR_RAMP[g] / 255.0,
        COLOR_RAMP[b] / 255.0,
    ];
    this.body_color = [
        COLOR_RAMP[r + 1] / 255.0,
        COLOR_RAMP[g + 1] / 255.0,
        COLOR_RAMP[b + 1] / 255.0,
    ];

    this.name = name;
}
exports.PonyAppearance = PonyAppearance;

PonyAppearance.prototype.buildSprite = function(pos, frame) {
    return new Sprite(this)
        .setSize(96, 96)
        .setRefPosition(pos.x, pos.y, pos.z)
        .setAnchor(48, 90)
        .setFrame(frame.sheet, frame.i, frame.j)
        .setFlip(frame.flip);
};

PonyAppearance.prototype.getClass = function() {
    return PonyAppearanceClass;
};

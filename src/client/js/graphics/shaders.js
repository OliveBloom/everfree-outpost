var CHUNK_SIZE = require('data/chunk').CHUNK_SIZE;
var TILE_SIZE = require('data/chunk').TILE_SIZE;

var sb = require('graphics/shaderbuilder');
var Uniforms = sb.Uniforms;
var Attributes = sb.Attributes;
var Textures = sb.Textures;



function makeShaders(shaders, gl, assets, defs, make_texture) {
    // TODO: hack
    var SIZEOF = window.SIZEOF;
    var ctx = new sb.ShaderBuilderContext(gl, assets, defs, make_texture);


    var square_buf = ctx.makeBuffer(new Uint8Array([
        -1, -1,
        -1,  1,
         1,  1,

        -1, -1,
         1,  1,
         1, -1,
    ]));

    var square01_buf = ctx.makeBuffer(new Uint8Array([
        0, 0,
        0, 1,
        1, 1,

        0, 0,
        1, 1,
        1, 0,
    ]));


    //
    // Terrain
    //

    shaders.terrain = ctx.start('terrain2.vert', 'terrain2.frag', 2)
        .uniformVec2('cameraPos')
        .uniformVec2('cameraSize')
        .uniformVec2('sliceCenter')
        .uniformFloat('sliceZ')
        .attributes(new Attributes(SIZEOF.TerrainVertex)
                .field(0, gl.UNSIGNED_BYTE, 2, 'corner')
                .field(2, gl.UNSIGNED_BYTE, 3, 'blockPos')
                .field(5, gl.UNSIGNED_BYTE, 1, 'side')
                .field(6, gl.UNSIGNED_BYTE, 2, 'tileCoord'))
        .texture('atlasTex', ctx.makeAssetTexture('tiles'))
        .texture('cavernTex', null)
        .finish();


    //
    // Light
    //

    var light_base = ctx.start('light2.vert', 'light2.frag', 1)
        .uniformVec2('cameraPos')
        .uniformVec2('cameraSize')
        .texture('depthTex');

    shaders.light_static = light_base.copy()
        .define('LIGHT_INPUT', 'attribute')
        .attributes(new Attributes(SIZEOF.LightVertex)
                .field( 0, gl.UNSIGNED_BYTE,  2, 'corner')
                .field( 2, gl.UNSIGNED_SHORT, 3, 'center')
                .field( 8, gl.UNSIGNED_BYTE,  3, 'colorIn', true)
                .field(12, gl.UNSIGNED_SHORT, 1, 'radiusIn'))
        .finish();

    shaders.light_dynamic = light_base.copy()
        .define('LIGHT_INPUT', 'uniform')
        .uniformVec3('center')
        .uniformVec3('colorIn')
        .uniformFloat('radiusIn')
        .attributes(new Attributes(2, square01_buf)
                .field( 0, gl.UNSIGNED_BYTE,  2, 'corner'))
        .finish();


    //
    // Structure
    //

    var structure_uniforms = new Uniforms()
        .vec2('cameraPos')
        .vec2('cameraSize')
        .vec2('sliceCenter')
        .float_('sliceZ')
        .float_('now');

    var structure_attributes = new Attributes(SIZEOF.StructureVertex)
            .field( 0, gl.UNSIGNED_SHORT, 3, 'vertOffset')
            .field( 6, gl.BYTE,           1, 'animLength')
            .field( 7, gl.UNSIGNED_BYTE,  1, 'animRate')
            .field( 8, gl.UNSIGNED_BYTE,  3, 'blockPos')
            .field(11, gl.UNSIGNED_BYTE,  1, 'layer')
            .field(12, gl.SHORT,          2, 'displayOffset')
            .field(16, gl.UNSIGNED_SHORT, 1, 'animOneshotStart')
            .field(18, gl.UNSIGNED_SHORT, 1, 'animStep');

    var structure_textures = new Textures()
        .texture('sheetTex', ctx.makeAssetTexture('structures0'))
        .texture('cavernTex', null);

    shaders.structure = ctx.start('structure2.vert', 'structure2.frag', 2)
        .uniforms(structure_uniforms)
        .attributes(structure_attributes)
        .textures(structure_textures)
        .finish();

    shaders.structure_shadow = ctx.start('structure2.vert', 'structure2.frag', 1)
        .define('OUTPOST_SHADOW', '1')
        .uniforms(structure_uniforms)
        .attributes(structure_attributes)
        .textures(structure_textures)
        .finish();


    //
    // Blits
    //

    var blit_attributes = new Attributes(2, square01_buf)
        .field(0, gl.UNSIGNED_BYTE, 2, 'posOffset');
    var blit_textures = new Textures()
        .texture('image0Tex')
        .texture('image1Tex')
        .texture('depthTex');

    shaders.blit_full = ctx.start('blit_fullscreen.vert', 'blit_output.frag', 1)
        .attributes(blit_attributes)
        .texture('imageTex')
        .finish();

    // TODO: hack
    if (!defs || !defs['SLICE_ENABLE']) {
        shaders.blend_layers = ctx.start('blit_fullscreen.vert', 'blend_layers.frag', 1)
            .uniformVec2('cameraPos')
            .uniformVec2('cameraSize')
            .uniformVec2('sliceCenter')
            .uniformFloat('sliceZ')
            .attributes(blit_attributes)
            .texture('baseTex')
            .texture('slicedTex')
            .texture('cavernTex')
            .finish();
    }

    shaders.post_filter = ctx.start('blit_fullscreen.vert', 'blit_post.frag', 1)
        .uniformVec2('screenSize')
        .attributes(blit_attributes)
        .textures(blit_textures)
        .texture('lightTex')
        .texture('shadowTex')
        .texture('shadowDepthTex')
        .finish();


    //
    // UI
    //

    var item_tex = ctx.makeAssetTexture('items_img');
    var ui_tex = ctx.makeAssetTexture('ui_atlas');
    var font_tex = ctx.makeAssetTexture('fonts');
    shaders.ui_blit2 = ctx.start('ui_blit2.vert', 'ui_blit2.frag')
        .uniformVec2('screenSize')
        // TODO: ugly hack - we lie and say the item texture is half as big as
        // it actually is, so that geometry generated with 16x16 icons in mind
        // will work with the actual 32x32-icon sheet
        .uniformVec2('sheetSize[0]', [item_tex.width / 2, item_tex.height / 2])
        .uniformVec2('sheetSize[1]', [ui_tex.width, ui_tex.height])
        .uniformVec2('sheetSize[2]', [font_tex.width, font_tex.height])
        .attributes(new Attributes(16)
                .field(0, gl.UNSIGNED_SHORT, 2, 'srcPos')
                .field(4, gl.UNSIGNED_BYTE, 2, 'srcSize')
                .field(6, gl.UNSIGNED_BYTE, 1, 'sheetAttr')
                .field(8, gl.SHORT, 2, 'dest')
                .field(12, gl.UNSIGNED_SHORT, 2, 'offset_'))
        .texture('sheets[0]', item_tex)
        .texture('sheets[1]', ui_tex)
        .texture('sheets[2]', font_tex)
        .finish();

}
exports.makeShaders = makeShaders;

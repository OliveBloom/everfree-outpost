var sb = require('graphics/shaderbuilder');

var Uniforms = sb.Uniforms;
var Attributes = sb.Attributes;
var Textures = sb.Textures;


function makeShaders(shaders, gl, assets, make_texture) {
    var ctx = new sb.ShaderBuilderContext(gl, assets, make_texture);


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
    // Terrain Block
    //

    var terrain_atlas = ctx.makeAssetTexture('tiles');
    shaders.terrain_block = ctx.start('terrain_block.vert', 'terrain_block.frag', 2)
        .uniformVec2('atlasSize', [(terrain_atlas.width / TILE_SIZE)|0,
                                   (terrain_atlas.height / TILE_SIZE)|0])
        .attributes(new Attributes(8)
                .field(0, gl.UNSIGNED_BYTE, 3, 'position')
                .field(3, gl.UNSIGNED_BYTE, 1, 'side')
                .field(4, gl.UNSIGNED_BYTE, 2, 'texCoord'))
        .texture('atlasTex', terrain_atlas)
        .finish();


    //
    // Blits
    //

    var blit_uniforms = new Uniforms()
        .vec2('rectPos')
        .vec2('rectSize', [CHUNK_SIZE * TILE_SIZE, CHUNK_SIZE * TILE_SIZE])
        .vec2('cameraPos')
        .vec2('cameraSize');
    var blit_attributes = new Attributes(2, square01_buf)
        .field(0, gl.UNSIGNED_BYTE, 2, 'posOffset');
    var blit_textures = new Textures()
        .texture('image0Tex')
        .texture('image1Tex')
        .texture('depthTex');

    shaders.blit = ctx.start('blit.vert', 'blit.frag', 2)
        .uniforms(blit_uniforms)
        .attributes(blit_attributes)
        .textures(blit_textures)
        .finish();

    shaders.blit_sliced = ctx.start('blit.vert', 'blit_sliced.frag', 2)
        .uniforms(blit_uniforms)
        .uniformFloat('sliceFrac')
        .attributes(blit_attributes)
        .texture('upperImage0Tex')
        .texture('upperImage1Tex')
        .texture('upperDepthTex')
        .texture('lowerImage0Tex')
        .texture('lowerImage1Tex')
        .texture('lowerDepthTex')
        .finish();

    shaders.blit_full = ctx.start('blit_fullscreen.vert', 'blit_output.frag', 1)
        .attributes(blit_attributes)
        .texture('imageTex')
        .finish();

    shaders.blit_depth = ctx.start('blit_fullscreen.vert', 'blit_depth.frag', 1)
        .attributes(blit_attributes)
        .texture('depthTex')
        .finish();

    shaders.post_filter = ctx.start('blit_fullscreen.vert', 'blit_post.frag', 1)
        .uniformVec2('screenSize')
        .attributes(blit_attributes)
        .textures(blit_textures)
        .texture('lightTex')
        .finish();


    //
    // Lights
    //

    var light_base = ctx.start('light.vert', 'light.frag', 1)
        .uniformVec2('cameraPos')
        .uniformVec2('cameraSize')
        .texture('depthTex');

    shaders.static_light = light_base.copy()
        .define('LIGHT_INPUT', 'attribute')
        .attributes(new Attributes(16)
                .field( 0, gl.BYTE,           2, 'posOffset')
                .field( 2, gl.SHORT,          3, 'center')
                .field( 8, gl.UNSIGNED_BYTE,  3, 'colorIn', true)
                .field(12, gl.UNSIGNED_SHORT, 1, 'radiusIn'))
        .finish();

    shaders.dynamic_light = light_base.copy()
        .define('LIGHT_INPUT', 'uniform')
        .uniformVec3('center')
        .uniformVec3('colorIn')
        .uniformFloat('radiusIn')
        .attributes(new Attributes(2, square_buf)
                .field( 0, gl.BYTE,           2, 'posOffset'))
        .finish();


    //
    // Structures
    //

    var struct_sheet = ctx.makeAssetTexture('structures0');
    var staticanim_sheet = ctx.makeAssetTexture('staticanim0');

    var struct_uniforms = new Uniforms()
        .vec2('sheetSize', [struct_sheet.width, struct_sheet.height]);
    var struct_attributes = new Attributes(16)
        .field( 0, gl.SHORT,          3, 'position')
        .field( 6, gl.SHORT,          1, 'baseZAttr')
        .field( 8, gl.UNSIGNED_SHORT, 2, 'texCoord');
    var struct_textures = new Textures()
        .texture('sheetTex', struct_sheet)
        .texture('depthTex', ctx.makeAssetTexture('structdepth0'));

    shaders.structure = ctx.start('structure.vert', 'structure.frag', 2)
        .uniforms(struct_uniforms)
        .attributes(struct_attributes)
        .textures(struct_textures)
        .finish();

    shaders.structure_shadow = ctx.start('structure.vert', 'structure_shadow.frag', 2)
        .uniforms(struct_uniforms)
        .attributes(struct_attributes)
        .textures(struct_textures)
        .finish();

    shaders.structure_anim = ctx.start('structure_anim.vert', 'structure.frag', 2)
        .uniformVec2('sheetSize', [staticanim_sheet.width, staticanim_sheet.height])
        .uniformFloat('now')
        .attributes(struct_attributes.copy()
                .field(13, gl.UNSIGNED_BYTE,  1, 'animRate')
                .field(14, gl.UNSIGNED_BYTE,  1, 'animLength')
                .field(15, gl.UNSIGNED_BYTE,  1, 'animStep'))
        .texture('sheetTex', staticanim_sheet)
        .texture('depthTex', ctx.makeAssetTexture('staticanimdepth0'))
        .finish();
}
exports.makeShaders = makeShaders;

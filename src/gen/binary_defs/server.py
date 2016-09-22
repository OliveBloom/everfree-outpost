from binary_defs.context import *


# Increment when an existing section's format is changed
VER_MINOR = 0


FILES = (
        'blocks',
        'items',
        'recipes',
        'structures',
        'animations',
        'sprite_layers',
        )


BLOCK = Struct((
    Field('name',           String()),
    Field('flags',          Scalar('H')),
    ))


ITEM = Struct((
    Field('name',           String()),
    ))


RECIPE_ITEM = Vector('H', 2)

RECIPE = Struct((
    Field('name',           String()),
    Field('inputs',         Sequence(b'RcpeItms', RECIPE_ITEM)),
    Field('outputs',        Sequence(b'RcpeItms', RECIPE_ITEM)),
    Field('ability',        Scalar('H')),
    Field('station',        Scalar('I')),
    ))


STRUCTURE_SHAPE = Scalar('H')

STRUCTURE = Struct((
    Field('name',           String()),
    Field('size',           Vector('i', 3)),
    Field('shape',          Sequence(b'StrcShap', STRUCTURE_SHAPE)),
    Field('layer',          Scalar('B')),
    ))


ANIMATION = Struct((
    Field('name',           String()),
    Field('framerate',      Scalar('B')),
    Field('length',         Scalar('B')),
    ))

SPRITE_LAYER = Struct((
    Field('name',           String()),
    ))


def convert(ctx, defs):
    ctx.init_intern_table(b'Strings\0', 1)
    ctx.init_intern_table(b'RcpeItms', RECIPE_ITEM.size())
    ctx.init_intern_table(b'StrcShap', STRUCTURE_SHAPE.size())

    ctx.convert(b'Blocks\0\0', BLOCK, defs['blocks'])
    ctx.convert(b'Items\0\0\0', ITEM, defs['items'])
    ctx.convert(b'RcpeDefs', RECIPE, defs['recipes'])
    ctx.convert(b'StrcDefs', STRUCTURE, defs['structures'])
    ctx.convert(b'SprtAnim', ANIMATION, defs['animations'])
    ctx.convert(b'SprtLayr', SPRITE_LAYER, defs['sprite_layers'])

    ctx.build_index(b'Blck', (x['name'] for x in defs['blocks']))
    ctx.build_index(b'Item', (x['name'] for x in defs['items']))
    ctx.build_index(b'Rcpe', (x['name'] for x in defs['recipes']))
    ctx.build_index(b'Strc', (x['name'] for x in defs['structures']))
    ctx.build_index(b'Anim', (x['name'] for x in defs['animations']))
    ctx.build_index(b'Layr', (x['name'] for x in defs['sprite_layers']))

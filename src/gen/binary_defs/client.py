from binary_defs.context import *


# Increment when an existing section's format is changed
VER_MINOR = 0


FILES = (
        'blocks',
        'items',
        'structures',
        'structure_parts',
        'structure_verts',
        'structure_shapes',
        'animations',
        'sprite_layers',
        'sprite_graphics',
        'recipes',
        'extras',
        'day_night',
        )


BLOCK = Struct((
    Field('front',          Scalar('H'),    default=0),
    Field('back',           Scalar('H'),    default=0),
    Field('top',            Scalar('H'),    default=0),
    Field('bottom',         Scalar('H'),    default=0),

    Field('light_r',        Scalar('B'),    default=0),
    Field('light_g',        Scalar('B'),    default=0),
    Field('light_b',        Scalar('B'),    default=0),
    Field('light_radius',   Scalar('H'),    default=0),

    Field('flags',          Scalar('H')),
    ))


ITEM = Struct((
    Field('name',           String()),
    Field('ui_name',        String()),
    Field('desc',           String()),
    ))


STRUCTURE_VERT = Scalar('H')

STRUCTURE_PART = Struct((
    Field('vert_idx',       Scalar('H')),
    Field('vert_count',     Scalar('H')),
    Field('offset',         Vector('h', 2)),
    Field('sheet',          Scalar('B')),
    Field('flags',          Scalar('B'),    default=0),

    Field('anim_length',    Scalar('b'), 0),
    Field('anim_rate',      Scalar('B'), 0),
    Field('anim_step',      Scalar('H'), 0),
    ))

STRUCTURE_SHAPE = Scalar('H')

STRUCTURE = Struct((
    Field('size',           Vector('B', 3)),
    Field('shape_idx',      Scalar('H')),
    Field('part_idx',       Scalar('H')),
    Field('part_count',     Scalar('B')),
    Field('vert_count',     Scalar('B')),
    Field('layer',          Scalar('B')),
    Field('flags',          Scalar('B'),    default=0),

    Field('light_pos',      Vector('B', 3), default=(0, 0, 0)),
    Field('light_color',    Vector('B', 3), default=(0, 0, 0)),
    Field('light_radius',   Scalar('H'),    default=0),
    ))

def convert_structure_parts(ctx, parts):
    parts = [x.copy() for x in parts]
    for x in parts:
        if 'anim_size' in x:
            x['anim_step'] = x['anim_size'][0]
        if x.get('anim_oneshot', False):
            x['anim_length'] = -x['anim_length']
    ctx.convert(b'StrcPart', STRUCTURE_PART, parts)




ANIMATION = Struct((
    Field('local_id',       Scalar('H')),
    Field('framerate',      Scalar('B')),
    Field('length',         Scalar('B')),
    ))

SPRITE_LAYER = Struct((
    Field('start',          Scalar('H')),
    Field('count',          Scalar('H')),
    ))

SPRITE_GRAPHICS = Struct((
    Field('src_offset',     Vector('H', 2)),
    Field('dest_offset',    Vector('H', 2)),
    Field('size',           Vector('H', 2)),
    Field('sheet',          Scalar('B')),
    Field('mirror',         Scalar('B')),
    ))


RECIPE_ITEM = Vector('H', 2)

RECIPE = Struct((
    Field('ui_name',        String()),
    Field('inputs',         Sequence(b'RcpeItms', RECIPE_ITEM, idx_ty='H')),
    Field('outputs',        Sequence(b'RcpeItms', RECIPE_ITEM, idx_ty='H')),
    Field('ability',        Scalar('H')),
    Field('station',        Scalar('I')),
    ))


DAY_NIGHT_PHASE = Struct((
    Field('start_time',     Scalar('H')),
    Field('end_time',       Scalar('H')),
    Field('start_color',    Scalar('B')),
    Field('end_color',      Scalar('B')),
    ))

DAY_NIGHT_COLOR = Vector('B', 3)

def convert_day_night(ctx, j):
    cut_0 = 0
    colors = [(255, 255, 255)]
    cut_1 = len(colors)
    colors.extend(reversed(j['sunset']))
    cut_2 = len(colors)
    colors.extend(j['sunrise'])
    cut_3 = len(colors)

    phases = [
            dict(   # day
                start_time=j['day_start'],
                end_time=j['day_end'],
                start_color=cut_0,
                end_color=cut_1),
            dict(   # sunset
                start_time=j['day_end'],
                end_time=j['night_start'],
                start_color=cut_1,
                end_color=cut_2 - 1),
            dict(   # night
                start_time=j['night_start'],
                end_time=j['night_end'],
                start_color=cut_2 - 1,
                end_color=cut_2),
            dict(   # sunrise
                start_time=j['night_end'],
                end_time=j['day_start'],
                start_color=cut_2,
                end_color=cut_3 - 1),
            ]

    ctx.convert(b'DyNtPhas', DAY_NIGHT_PHASE, phases)
    ctx.convert(b'DyNtColr', DAY_NIGHT_COLOR, colors)


def convert_extras(ctx, j):
    # XPonLayr - pony_layer_table
    ctx.convert(b'XPonLayr', Scalar('B'), j['pony_layer_table'])

    # XPhysAnm - physics_anim_table
    arr = j['physics_anim_table']
    for i in range(len(arr)):
        if arr[i] is None:
            arr[i] = [0] * 8
    ctx.convert(b'XPhysAnm', Vector('H', 8), arr)

    # XAnimDir - anim_dir_table
    max_idx = max(int(k) for k in j['anim_dir_table'].keys())
    lst = [255] * (max_idx + 1)
    for k,v in j['anim_dir_table'].items():
        lst[int(k)] = v
    ctx.convert(b'XAnimDir', Scalar('B'), lst)

    # XSpcAnim - special anims
    ctx.convert(b'XSpcAnim', Scalar('H'), [
        j['default_anim'],
        j['editor_anim'],
        j['activity_none_anim'],
        ])

    # XSpcLayr - special layers
    ctx.convert(b'XSpcLayr', Scalar('B'), [
        j['activity_layer'],
        ])

    # XSpcGrfx - special graphics
    ctx.convert(b'XSpcGrfx', Scalar('H'), [
        j['activity_bubble_graphics'],
        ])



def convert(ctx, defs):
    ctx.init_intern_table(b'Strings\0', 1)
    ctx.init_intern_table(b'RcpeItms', RECIPE_ITEM.size())

    ctx.convert(b'Blocks\0\0', BLOCK, defs['blocks'])
    ctx.convert(b'Items\0\0\0', ITEM, defs['items'])
    ctx.convert(b'StrcVert', STRUCTURE_VERT, defs['structure_verts'])
    convert_structure_parts(ctx, defs['structure_parts'])
    ctx.convert(b'StrcShap', STRUCTURE_SHAPE, defs['structure_shapes'])
    ctx.convert(b'StrcDefs', STRUCTURE, defs['structures'])
    ctx.convert(b'SprtAnim', ANIMATION, defs['animations'])
    ctx.convert(b'SprtLayr', SPRITE_LAYER, defs['sprite_layers'])
    ctx.convert(b'SprtGrfx', SPRITE_GRAPHICS, defs['sprite_graphics'])
    ctx.convert(b'RcpeDefs', RECIPE, defs['recipes'])
    convert_day_night(ctx, defs['day_night'])
    convert_extras(ctx, defs['extras'])

    ctx.build_index(b'Item', (x['name'] for x in defs['items']))

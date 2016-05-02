from collections import namedtuple
import json
import os


from outpost_data.core import boxpack, builder2, files, image2, loader, util
from outpost_data.core import structure, block, item, recipe, sprite, loot_table, extra
from outpost_data.core.loader import TimeIt


Defs = namedtuple('Defs', (
    'blocks',
    'structures',
    'items',
    'recipes',
    'sprites',
    'loot_tables',
    'extras',
))

IdMaps = namedtuple('IdMaps', (
    'structures',
    'blocks',
    'items',
    'recipes',
    'sprites',
))

def collect_defs():
    return Defs(
            builder2.BLOCK.all(),
            builder2.STRUCTURE.all(),
            builder2.ITEM.all(),
            builder2.RECIPE.all(),
            builder2.SPRITE.all(),
            builder2.LOOT_TABLE.all(),
            builder2.EXTRA.all(),
            )

def postprocess(defs):
    id_maps = IdMaps(
        util.assign_ids(defs.structures),
        util.assign_ids(defs.blocks, ['empty', 'placeholder']),
        util.assign_ids(defs.items, ['none']),
        util.assign_ids(defs.recipes),
        util.assign_ids(defs.sprites),
    )

    recipe.resolve_item_ids(defs.recipes, id_maps.items)
    recipe.resolve_structure_ids(defs.recipes, id_maps.structures)
    sprite.process(defs.sprites)
    loot_table.resolve_object_ids(defs.loot_tables, id_maps)

    def_dicts = Defs(*({obj.name: obj for obj in x} for x in defs))
    extra.resolve_all(defs.extras, def_dicts)

def write_json(output_dir, basename, j):
    with open(os.path.join(output_dir, basename), 'w') as f:
        json.dump(j, f)

def emit_structures(output_dir, structures):
    # Final processing to assign array slots to parts, vertices, and shapes
    parts = structure.collect_parts(structures)
    verts = structure.collect_verts(parts)
    shapes = structure.collect_shapes(structures)

    # Handle sheet images
    for f in os.listdir(output_dir):
        if f.startswith('structures') and f.endswith('.png'):
            os.remove(os.path.join(output_dir, f))

    sheet_names = set()
    sheets = structure.build_sheets(structures)
    for i, image in enumerate(sheets):
        sheet_names.update(('structures%d' % i,))
        image.save(os.path.join(output_dir, 'structures%d.png' % i))

    write_json(output_dir, 'structures_list.json', sorted(sheet_names))

    # Handle parts and vertices
    write_json(output_dir, 'structure_parts_client.json',
            structure.build_parts_json(parts))

    write_json(output_dir, 'structure_verts_client.json',
            structure.build_verts_json(verts))

    write_json(output_dir, 'structure_shapes_client.json',
            structure.build_shapes_json(shapes))

    # Emit actual structure json
    write_json(output_dir, 'structures_server.json',
            structure.build_server_json(structures))

    write_json(output_dir, 'structures_client.json',
            structure.build_client_json(structures))

def emit_blocks(output_dir, blocks):
    sheet = block.build_sheet(blocks)
    sheet.save(os.path.join(output_dir, 'tiles.png'))

    write_json(output_dir, 'blocks_server.json',
            block.build_server_json(blocks))

    write_json(output_dir, 'blocks_client.json',
            block.build_client_json(blocks))

def emit_items(output_dir, items):
    sheet = item.build_sheet(items)
    sheet.save(os.path.join(output_dir, 'items.png'))

    write_json(output_dir, 'items_server.json',
            item.build_server_json(items))

    write_json(output_dir, 'items_client.json',
            item.build_client_json(items))

def emit_recipes(output_dir, recipes):
    write_json(output_dir, 'recipes_server.json',
            recipe.build_server_json(recipes))

    write_json(output_dir, 'recipes_client.json',
            recipe.build_client_json(recipes))

def emit_sprites(output_dir, sprites):
    # Build sheets first, to populate the offset fields
    packer = boxpack.ImagePacker((2048, 2048), res=16)
    # Place the name font first so it always sits at (0,0) on sheet 0
    # TODO: bit of a hack
    font_off = packer.place([image2.Image.open(os.path.join(output_dir, 'fonts/name.png'))])
    assert font_off == [(0, (0, 0))]
    sprite.pack_images(sprites, packer)
    anims, layers, graphics = sprite.collect_defs(sprites)

    # Write json
    write_json(output_dir, 'animations_server.json',
            sprite.build_anim_server_json(anims))

    write_json(output_dir, 'animations_client.json',
            sprite.build_anim_client_json(anims))

    write_json(output_dir, 'sprite_layers_client.json',
            sprite.build_layer_client_json(layers))

    write_json(output_dir, 'sprite_layers_server.json',
            sprite.build_layer_server_json(layers))

    write_json(output_dir, 'sprite_graphics_client.json',
            sprite.build_graphics_client_json(graphics))

    # Save sheets
    sprite_dir = os.path.join(output_dir, 'sprites')
    if not os.path.isdir(sprite_dir):
        os.mkdir(sprite_dir)

    sprite_list = []
    for i, img in enumerate(packer.build_sheets()):
        name = 'sprites%d.png' % i
        sprite_list.append(name)
        img.raw().raw().save(os.path.join(sprite_dir, name))

    write_json(output_dir, 'sprites_list.json', sprite_list)

def emit_loot_tables(output_dir, loot_tables):
    write_json(output_dir, 'loot_tables_server.json',
            loot_table.build_server_json(loot_tables))

def emit_extras(output_dir, extras):
    write_json(output_dir, 'extras_client.json',
            extra.build_client_json(extras))

def time(msg, f, *args):
    with TimeIt('  %s' % msg):
        f(*args)

def generate(output_dir):
    defs = collect_defs()
    postprocess(defs)

    print('Generating:')
    time('structures', emit_structures, output_dir, defs.structures)
    time('blocks', emit_blocks, output_dir, defs.blocks)
    time('items', emit_items, output_dir, defs.items)
    time('recipes', emit_recipes, output_dir, defs.recipes)
    time('sprites', emit_sprites, output_dir, defs.sprites)
    time('loot_tables', emit_loot_tables, output_dir, defs.loot_tables)
    time('extras', emit_extras, output_dir, defs.extras)

    print('%d structures, %d blocks, %d items, %d recipes' %
            (len(defs.structures), len(defs.blocks), len(defs.items), len(defs.recipes)))
    print('%d sprites, %d loot tables, %d extras' %
            (len(defs.sprites), len(defs.loot_tables), len(defs.extras)))

    with open(os.path.join(output_dir, 'stamp'), 'w') as f:
        pass

    with open(os.path.join(output_dir, 'used_assets.txt'), 'w') as f:
        f.write(''.join(p + '\n' for p in files.get_dependencies()))

    # Compute dependencies
    with open(os.path.join(output_dir, 'data.d'), 'w') as f:
        f.write('%s: \\\n' % os.path.join(output_dir, 'stamp'))
        for p in files.get_dependencies() + loader.get_dependencies():
            f.write('    %s \\\n' % p)

    assert not util.SAW_ERROR


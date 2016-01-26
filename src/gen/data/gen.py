from collections import namedtuple
import json
import os


from . import builder, builder2, files, loader, util
from . import structure, block, item, recipe, animation, attachment, loot_table, extra
from outpost_data.core.loader import TimeIt


Defs = namedtuple('Defs', (
    'blocks',
    'structures',
    'items',
    'recipes',
    'animations',
    'sprites',
    'attach_slots',
    'loot_tables',
    'extras',
))

IdMaps = namedtuple('IdMaps', (
    'structures',
    'blocks',
    'items',
    'recipes',
    'animations',
    'attach_slots',
    'attachments_by_slot',
))

def collect_defs(b):
    return Defs(
            builder2.BLOCK.all(),
            builder2.STRUCTURE.all(),
            builder2.ITEM.all(),
            builder2.RECIPE.all(),
            b.animations,
            b.sprites,
            b.attach_slots,
            builder2.LOOT_TABLE.all(),
            builder2.EXTRA.all(),
            )

def postprocess(defs):
    id_maps = IdMaps(
        util.assign_ids(defs.structures),
        util.assign_ids(defs.blocks, ['empty', 'placeholder']),
        util.assign_ids(defs.items, ['none']),
        util.assign_ids(defs.recipes),
        util.assign_ids(defs.animations),
        util.assign_ids(defs.attach_slots),
        dict((s.name, util.assign_ids(s.variants, ['none'])) for s in defs.attach_slots),
    )

    recipe.resolve_item_ids(defs.recipes, id_maps.items)
    recipe.resolve_structure_ids(defs.recipes, id_maps.structures)
    loot_table.resolve_object_ids(defs.loot_tables, id_maps)
    extra.resolve_all(defs.extras, id_maps)

def write_json(output_dir, basename, j):
    with open(os.path.join(output_dir, basename), 'w') as f:
        json.dump(j, f)

def emit_structures(output_dir, structures):
    # Final processing to assign array slots to parts and vertices
    parts = structure.collect_parts(structures)
    verts = structure.collect_verts(parts)

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

def emit_animations(output_dir, animations):
    write_json(output_dir, 'animations_server.json',
            animation.build_server_json(animations))

    write_json(output_dir, 'animations_client.json',
            animation.build_client_json(animations))

def emit_sprites(output_dir, sprites):
    os.makedirs(os.path.join(output_dir, 'sprites'), exist_ok=True)

    sprite_names = set()
    for s in sprites:
        if s.name in sprite_names:
            util.err('duplicate sprite definition: %r' % s.name)

        for i, img in enumerate(s.images):
            basename = '%s-%d' % (s.name.replace('/', '_'), i)
            sprite_names.add(basename)
            img.save(os.path.join(output_dir, 'sprites', basename + '.png'))

    write_json(output_dir, 'sprites_list.json', sorted(sprite_names))

def emit_attach_slots(output_dir, attach_slots):
    write_json(output_dir, 'attach_slots_server.json',
            attachment.build_server_json(attach_slots))

    write_json(output_dir, 'attach_slots_client.json',
            attachment.build_client_json(attach_slots))

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
    b = builder.INSTANCE
    defs = collect_defs(b)
    postprocess(defs)

    print('Generating:')
    time('structures', emit_structures, output_dir, defs.structures)
    time('blocks', emit_blocks, output_dir, defs.blocks)
    time('items', emit_items, output_dir, defs.items)
    time('recipes', emit_recipes, output_dir, defs.recipes)
    time('animations', emit_animations, output_dir, defs.animations)
    time('sprites', emit_sprites, output_dir, defs.sprites)
    time('attach_slots', emit_attach_slots, output_dir, defs.attach_slots)
    time('loot_tables', emit_loot_tables, output_dir, defs.loot_tables)
    time('extras', emit_extras, output_dir, defs.extras)

    print('%d structures, %d blocks, %d items, %d recipes' %
            (len(defs.structures), len(defs.blocks), len(defs.items), len(defs.recipes)))
    print('%d animations, %d sprites, %d attach_slots, %d loot tables, %d extras' %
            (len(defs.animations), len(defs.sprites), len(defs.attach_slots),
                len(defs.loot_tables), len(defs.extras)))

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


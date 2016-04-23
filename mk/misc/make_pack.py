from itertools import count
import json
import os
import struct
import sys

def main(src_dir, build_dir, out_file):
    index = []
    paths = []
    hidden_deps = set()

    src = lambda path: os.path.join(src_dir, path)
    build = lambda path: os.path.join(build_dir, path)

    def add(ty, name, path, hide_dep=False):
        size = os.stat(path).st_size

        index.append({
                'name': name,
                'length': size,
                'type': ty,
                })
        paths.append(path)

        if hide_dep:
            hidden_deps.add(path)

    add('image', 'tiles',       build('tiles.png'))
    add('image', 'fonts',       build('fonts.png'))
    add('url',   'items',       build('items.png'))
    add('image', 'items_img',   build('items.png'))
    add('image', 'ui_atlas',    build('ui_atlas.png'))

    add('json', 'item_defs',            build('items_client.json'))
    add('json', 'recipe_defs',          build('recipes_client.json'))
    add('json', 'animation_defs',       build('animations_client.json'))
    add('json', 'extra_defs',           build('extras_client.json'))
    add('json', 'fonts_metrics',        build('fonts_metrics.json'))
    add('json', 'day_night',            build('day_night.json'))
    add('json', 'ui_atlas_parts',       build('ui_atlas.json'))

    add('binary', 'block_defs_bin',             build('blocks_client.bin'))
    add('binary', 'item_defs_bin',              build('item_defs_client.bin'))
    add('binary', 'item_strs_bin',              build('item_strs_client.bin'))
    add('binary', 'template_defs_bin',          build('structures_client.bin'))
    add('binary', 'template_part_defs_bin',     build('structure_parts_client.bin'))
    add('binary', 'template_vert_defs_bin',     build('structure_verts_client.bin'))
    add('binary', 'template_shape_defs_bin',    build('structure_shapes_client.bin'))
    add('binary', 'animation_defs_bin',         build('animations_client.bin'))
    add('binary', 'sprite_layer_defs_bin',      build('sprite_layers_client.bin'))
    add('binary', 'sprite_graphics_defs_bin',   build('sprite_graphics_client.bin'))
    add('binary', 'extras_bin',                 build('extras_client.bin'))

    add('text', 'sprite.vert',          src('assets/shaders/sprite.vert'))
    add('text', 'sprite.frag',          src('assets/shaders/sprite.frag'))
    add('text', 'app_pony.frag',        src('assets/shaders/app_pony.frag'))
    add('text', 'cursor.frag',          src('assets/shaders/cursor.frag'))
    add('text', 'cursor.vert',          src('assets/shaders/cursor.vert'))

    add('text', 'blit_post.frag',       src('assets/shaders/blit_post.frag'))
    add('text', 'blit_output.frag',     src('assets/shaders/blit_output.frag'))
    add('text', 'blend_layers.frag',    src('assets/shaders/blend_layers.frag'))
    add('text', 'blit_fullscreen.vert', src('assets/shaders/blit_fullscreen.vert'))

    add('text', 'terrain2.frag',        src('assets/shaders/terrain2.frag'))
    add('text', 'terrain2.vert',        src('assets/shaders/terrain2.vert'))
    add('text', 'structure2.frag',      src('assets/shaders/structure2.frag'))
    add('text', 'structure2.vert',      src('assets/shaders/structure2.vert'))
    add('text', 'light2.frag',          src('assets/shaders/light2.frag'))
    add('text', 'light2.vert',          src('assets/shaders/light2.vert'))
    add('text', 'entity2.frag',         src('assets/shaders/entity2.frag'))
    add('text', 'entity2.vert',         src('assets/shaders/entity2.vert'))
    add('text', 'slicing.inc',          src('assets/shaders/slicing.inc'))

    add('text', 'debug_graph.vert',     src('assets/shaders/debug_graph.vert'))
    add('text', 'debug_graph.frag',     src('assets/shaders/debug_graph.frag'))

    add('text', 'ui_blit.vert',         src('assets/shaders/ui_blit.vert'))
    add('text', 'ui_blit.frag',         src('assets/shaders/ui_blit.frag'))
    add('text', 'ui_blit_tiled.vert',   src('assets/shaders/ui_blit_tiled.vert'))
    add('text', 'ui_blit_tiled.frag',   src('assets/shaders/ui_blit_tiled.frag'))
    add('text', 'ui_blit2.vert',        src('assets/shaders/ui_blit2.vert'))
    add('text', 'ui_blit2.frag',        src('assets/shaders/ui_blit2.frag'))


    with open(build('structures_list.json')) as f:
        structures_list = json.load(f)
    for s in structures_list:
        add('image', s, build(s + '.png'))

    with open(build('sprites_list.json')) as f:
        sprites_list = json.load(f)
    for f in sprites_list:
        dest, _ = os.path.splitext(os.path.basename(f))
        add('image', dest, build(os.path.join('sprites', f)))


    # Generate the pack containing the files added above.

    offset = 0
    for entry in index:
        entry['offset'] = offset
        offset += entry['length']


    index_str = json.dumps(index)
    index_len = len(index_str.encode())

    with open(out_file, 'wb') as f:
        f.write(struct.pack('<I', len(index_str.encode())))
        f.write(index_str.encode())

        for (entry, path) in zip(index, paths):
            total_len = 0
            with open(path, 'rb') as f2:
                while True:
                    chunk = f2.read(4096)
                    f.write(chunk)
                    total_len += len(chunk)
                    if len(chunk) == 0:
                        break

            assert total_len == entry['length'], \
                    'file %r changed length during packing' % entry['name']

    # Emit dependencies
    with open(out_file + '.d', 'w') as f:
        f.write('%s: \\\n' % out_file)
        for path in paths:
            if path in hidden_deps:
                continue
            f.write('    %s \\\n' % path)

if __name__ == '__main__':
    src_dir, build_dir, out_file = sys.argv[1:]
    main(src_dir, build_dir, out_file)

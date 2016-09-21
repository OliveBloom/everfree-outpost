import os

from configure.template import template
from configure.util import join, maybe


def rules(i):
    return template('''
        rule process_font
            command = $python3 $root/src/gen/process_font.py $
                --font-image-in=$in $
                $extra_args $
                --font-image-out=$out_img $
                --font-metrics-out=$out_metrics
            description = GEN $out_img

        rule stack_fonts
            command = $python3 $root/src/gen/stack_fonts.py $
                $out_img $out_metrics $out_rust $in
            description = GEN $out_img

        rule process_day_night
            command = $python3 $root/src/gen/gen_day_night.py $in >$out
            description = GEN $out

        rule gen_ui_atlas
            command = $python3 $root/src/gen/gen_ui_atlas.py $in_dir $out_dir
            description = GEN $out_dir/ui_atlas.png
            depfile = $out_dir/ui_atlas.d

        rule gen_server_json
            command = $python3 $root/src/gen/gen_server_json.py >$out
            description = GEN $out

        rule gen_credits
            command = $python3 $root/src/gen/gen_credits.py $root $out $dep_files
            description = GEN $out
    ''', **locals())

def font(basename, src_img, charset_args='--first-char=0x21', extra_args=''):
    out_img = '$b_data/fonts/' + basename + '.png'
    out_metrics = '$b_data/fonts/' + basename + '_metrics.json'

    return template('''
        build %out_img %out_metrics: process_font %src_img $
            | $root/src/gen/process_font.py
            extra_args = %charset_args %extra_args
            out_img = %out_img
            out_metrics = %out_metrics
    ''', **locals())

def font_stack(out_base, in_basenames):
    out_img = out_base + '.png'
    out_metrics = out_base + '_metrics.json'
    out_rust = out_base + '_metrics.rs'

    return template('''
        build %out_img %out_metrics %out_rust: stack_fonts $
            %for name in in_basenames
                $b_data/fonts/%name.png $
                $b_data/fonts/%{name}_metrics.json $
            %end
            | $root/src/gen/stack_fonts.py
            out_img = %out_img
            out_metrics = %out_metrics
            out_rust = %out_rust
    ''', **locals())

def server_json(out_json):
    return template('''
        build %out_json: gen_server_json | $root/src/gen/gen_server_json.py
    ''', **locals())

def day_night(out_json, src_img):
    return template('''
        build %out_json: process_day_night %src_img $
            | $root/src/gen/gen_day_night.py
    ''', **locals())

def ui_atlas(out_dir, src_dir):
    return template('''
        build %out_dir/ui_atlas.png $
            %out_dir/ui_atlas.json $
            %out_dir/ui_atlas.rs: gen_ui_atlas $
            | $root/src/gen/gen_ui_atlas.py %src_dir
            in_dir = %src_dir
            out_dir = %out_dir
    ''', **locals())

def process():
    data_files = ['%s_%s.json' % (f,s)
            for s in ('server', 'client')
            for f in ('structures', 'blocks', 'items', 'recipes',
                'animations', 'sprite_layers')]
    data_files.append('structure_parts_client.json')
    data_files.append('structure_verts_client.json')
    data_files.append('structure_shapes_client.json')
    data_files.append('sprite_graphics_client.json')
    data_files.append('loot_tables_server.json')
    data_files.append('extras_client.json')

    # `sprites0.png` is explicitly mentioned in the manifest, to be copied to
    # the uvedit directory.
    data_files.append('sprites/sprites0.png')

    deps = [
            '$root/src/gen/data_main.py',
            # Name font is embedded into the sprite sheet
            '$b_data/fonts/name.png',

            # Native modules
            '$b_native/equip_sprites_render$_so',
            ]

    return template('''
        rule process_data
            command = $python3 $root/src/gen/data_main.py --mods=$mods $
                    --src-dir=$root --output-dir=$b_data $
                    --native-lib-dir=$b_native
            description = DATA
            depfile = $b_data/data.d

        build $b_data/stamp $
            %for name in data_files
                $b_data/%{name} $
            %end
            $b_data/tiles.png $b_data/items.png: $
            process_data | %{' '.join(deps)}
    ''', **locals())

def binary_defs(out_file):
    out_file = out_file or os.path.splitext(src_file)[0] + '.bin'

    deps = (
            '$b_data/stamp',
            '$b_data/day_night_client.json',
            )

    return template('''
        rule gen_binary_defs
            command = $python3 $root/src/gen/gen_binary_defs.py $
                    --gen-phf=$b_native/gen_phf client $b_data $out
            description = GEN $out
            depfile = $out.d

        build %out_file: gen_binary_defs $
            | $root/src/gen/gen_binary_defs.py $
              $b_native/gen_phf $
              %{' '.join(deps)}
    ''', **locals())

def pack():
    extra_data = (
            'fonts.png', 'fonts_metrics.json',
            'day_night_client.json',
            'ui_atlas.png', 'ui_atlas.json',
            'client_data.bin',
            )

    return template('''
        rule build_pack
            command = $python3 $root/mk/misc/make_pack.py $root $b_data $b_data/outpost.pack
            description = PACK
            depfile = $b_data/outpost.pack.d

        build $b_data/outpost.pack: build_pack $
            | $root/mk/misc/make_pack.py $
                %for name in extra_data
                    $b_data/%{name} $
                %end
            || $b_data/stamp
    ''', **locals())

def credits(out_path):
    return template('''
        build %out_path: gen_credits $
            | $b_data/stamp $b_data/outpost.pack $
              $root/src/gen/gen_credits.py
            dep_files = $b_data/data.d $b_data/outpost.pack.d
    ''', **locals())

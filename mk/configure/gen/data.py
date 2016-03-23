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
                $out_img $out_metrics $in
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

        rule gen_binary_defs
            command = $python3 $root/src/gen/gen_binary_defs.py --mode $mode $in $out
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

    return template('''
        build %out_img %out_metrics: stack_fonts $
            %for name in in_basenames
                $b_data/fonts/%name.png $
                $b_data/fonts/%{name}_metrics.json $
            %end
            | $root/src/gen/stack_fonts.py
            out_img = %out_img
            out_metrics = %out_metrics
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
        build %out_dir/ui_atlas.png %out_dir/ui_atlas.json: gen_ui_atlas $
            | $root/src/gen/gen_ui_atlas.py %src_dir
            in_dir = %src_dir
            out_dir = %out_dir
    ''', **locals())

def binary_defs(src_file, mode):
    out_file = os.path.splitext(src_file)[0] + '.bin'

    return template('''
        build %out_file: gen_binary_defs %src_file $
            | $root/src/gen/gen_binary_defs.py
            mode = %mode
    ''', **locals())

def process():
    data_files = ['%s_%s.json' % (f,s)
            for s in ('server', 'client')
            for f in ('structures', 'blocks', 'items', 'recipes', 'animations', 'sprite_parts')]
    data_files.append('loot_tables_server.json')
    data_files.append('extras_client.json')
    return template('''
        rule process_data
            command = $python3 $root/src/gen/data_main.py --mods=$mods $
                    --src-dir=$root --output-dir=$b_data
            description = DATA
            depfile = $b_data/data.d

        build $b_data/stamp $
            %for name in data_files
                $b_data/%{name} $
            %end
            $b_data/tiles.png $b_data/items.png: $
            process_data | $root/src/gen/data_main.py
    ''', **locals())

def pack():
    extra_data = (
            'fonts.png', 'fonts_metrics.json',
            'day_night.json',
            'ui_atlas.png', 'ui_atlas.json',
            ) + tuple('%s_client.bin' % f
                    for f in ('blocks', 'structures', 'structure_parts',
                        'structure_verts', 'structure_shapes'))

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

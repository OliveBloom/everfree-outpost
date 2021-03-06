import os

from configure.template import template
from configure.util import join, maybe


def rules(i):
    return template('''
        rule scripts_stamp
            command = touch $out
            description = STAMP $out

        rule copy_mod_scripts
            command = $python3 $root/mk/misc/copy_mod_scripts.py $
                --mod-name $mod_name $
                --input-dir $input_dir $
                --output-dir $output_dir $
                --stamp $out $
                --ext $ext
            description = COPY $output_dir/$mod_name ($out)
            depfile = $out.d

        rule copy_script
            command = cp -f $in $out
            description = COPY $out
    ''', **locals())

def copy_mod_scripts(mods):
    builds = []
    def add_build(*args, **kwargs):
        builds.append(template(*args, **kwargs))

    for mod in mods:
        if mod != 'outpost':
            input_dir2 = os.path.join('$root', 'mods', mod, 'scripts')
        else:
            input_dir2 = os.path.join('$root', 'scripts', 'outpost')

        add_build('''
            build $b_scripts/stamp/%mod: copy_mod_scripts $
                    | $root/mk/misc/copy_mod_scripts.py
                mod_name = %mod
                input_dir = %input_dir2
                output_dir = $b_scripts/gen
                ext = py
        ''', **locals())

    add_build('''
        build $b_scripts/stamp/core: copy_mod_scripts $
                | $root/mk/misc/copy_mod_scripts.py
            mod_name = core
            input_dir = $root/scripts/core
            output_dir = $b_scripts/gen
            ext = py

        build $b_scripts/gen/boot.py: copy_script $root/scripts/boot.py

        build $b_scripts/gen/: scripts_stamp | $
            %for mod in mods
            $b_scripts/stamp/%mod $
            %end
            $b_scripts/gen/boot.py $
            $b_scripts/stamp/core
    ''', **locals())

    return '\n\n'.join(builds)

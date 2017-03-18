import os

from configure.template import template
from configure.util import cond, join, maybe, mk_build


def rules(i):

    return template('''
        # This rule uses a separate `$in_exact` variable, which contains the
        # pre-normalization version of `$in`.  This is necessary for the paths
        # to line up with `$module_dir`.
        rule js_compile_modules
            command = $
                %if not i.debug
                $closure_compiler $
                    $$($python3 $root/mk/misc/collect_js_deps.py $in_exact $out $depfile) $
                    --js_output_file=$out $
                    --language_in=ECMASCRIPT5_STRICT $
                    --compilation_level=ADVANCED_OPTIMIZATIONS $
                    --output_wrapper='(function(){%{}output%{}})();' $
                    --jscomp_error=undefinedNames $
                    --jscomp_error=undefinedVars $
                    --create_name_map_files $
                    --process_common_js_modules $
                    --common_js_entry_module=$entry_module $
                    --common_js_module_path_prefix=$module_dir $
                    --externs=$root/mk/misc/closure_externs.js
                %else
                $python3 $root/mk/misc/gen_js_loader.py $
                    $$($python3 $root/mk/misc/collect_js_deps.py $in $out $depfile) $
                    >$out
                %end
            description = MIN $out
            depfile = $out.d

        rule js_minify_file
            command = $
                %if not i.debug
                $python3 $root/mk/misc/minify.py <$in >$out
                %else
                cp $in $out
                %end
            description = MIN $out
    ''', **locals())

def compile(i, out_file, main_src):
    main_dir, basename_ext = os.path.split(main_src)
    module_name, _ = os.path.splitext(basename_ext)

    return template('''
        build %out_file: js_compile_modules %main_src $
            | $root/mk/misc/collect_js_deps.py $
              %if i.debug% $root/mk/misc/gen_js_loader.py %end%
            entry_module = %module_name
            module_dir = %main_dir
            in_exact = %main_src
    ''', **locals())

def minify(i, out_file, js_src):
    if out_file is None:
        out_file = '$b_js/%s' % os.path.basename(js_src)

    return template('''
        build %out_file: js_minify_file %js_src $
            | %if i.debug% $root/mk/misc/minify.py %end%
            filter = | sed -e '1s/{/{"use asm";/'
    ''', **locals())

def client_manifest(i, out_file):
    return template('''
        rule gen_client_manifest
            command = $python3 $root/mk/misc/gen_client_manifest.py $
                --file $b_js/asmlibs.js $
                --file $root/src/client/client_parts.html $
                %if not i.debug
                --file $b_js/outpost.js $
                %else
                --walk-js-file $root/src/client/js/main.js::js $
                %end
                --output $out
            description = GEN $out
            depfile = $out.d

        build %out_file: gen_client_manifest $
            | $root/mk/misc/gen_client_manifest.py $
              $b_js/asmlibs.js $
              %if not i.debug% $b_js/outpost.js %end%
    ''', **locals())

import argparse
import os
import subprocess
import sys

from configure import checks
from configure.gen import native, asmjs, data, js, dist, scripts, www
from configure.template import template


def build_parser():
    args = argparse.ArgumentParser()

    args.add_argument('--build-dir', default=None,
            help='directory to store build files')
    args.add_argument('--dist-dir', default=None,
            help='directory to store distribution image')

    args.add_argument('--reconfigure', action='store_true', default=False,
            help='reuse cached configuration info when possible')

    args.add_argument('--components', default='all',
            help='list of components to build')
    args.add_argument('--mods',
            help='list of mods to include in the compiled game')

    args.add_argument('--debug', action='store_true', default=False,
            help='produce a debug build')
    args.add_argument('--release', action='store_false', dest='debug',
            help='produce a release build (default)')

    args.add_argument('--rust-home', default='../rust',
            help='path to rust-lang/rust checkout')
    args.add_argument('--bitflags-home', default='../bitflags',
            help='path to rust-lang/bitflags checkout')
    args.add_argument('--rust-extra-libdir', default=None,
            help='additional search directory for Rust libraries')
    args.add_argument('--rust-lib-externs', default='',
            help='list of --extern flags for locating Rust libraries')
    args.add_argument('--emscripten-fastcomp-prefix', default=None,
            help='path to kripken/emscripten-fastcomp build/install directory')

    args.add_argument('--rustc',
            help='name of the Rust compiler binary')
    args.add_argument('--cc',
            help='name of the C compiler binary')
    args.add_argument('--cxx',
            help='name of the C++ compiler binary')
    args.add_argument('--python3',
            help='name of the Python 3 interpreter binary')
    args.add_argument('--python3-config',
            help='name of the Python 3 build configuration helper binary')
    args.add_argument('--closure-compiler',
            help='name of the Closure Compiler binary')
    args.add_argument('--yui-compressor',
            help='name of the YUI Compressor binary')

    args.add_argument('--force', action='store_true', default=False,
            help='proceed even if there are configuration errors')

    args.add_argument('--cflags',
            help='extra flags for the C compiler')
    args.add_argument('--cxxflags',
            help='extra flags for the C++ compiler')
    args.add_argument('--ldflags',
            help='extra flags for the C/C++ linker')

    args.add_argument('--site-config',
            help='YAML file containing site-specific config')

    return args


def parse_components(s):
    parts = s.split(',')


def header(i):
    def b(*args):
        return os.path.normpath(os.path.join(i.build_dir, *args))

    return template('''
        # Root of the source tree.  This used to be called $src, but that would
        # be confusing now that $root/src is an actual directory.
        root = %{os.path.normpath(i.root_dir)}
        # Note: (1) `build` is a ninja keyword; (2) `builddir` is a special
        # variable that determines where `.ninja_log` is stored.
        builddir = %{os.path.normpath(i.build_dir)}
        dist = %{os.path.normpath(i.dist_dir)}

        _exe = %{'' if not i.win32 else '.exe'}
        _so = %{'.so' if not i.win32 else '.dll'}
        _a = .a

        b_native = %{b('native')}
        b_asmjs = %{b('asmjs')}
        b_data = %{b('data')}
        b_js = %{b('js')}
        b_scripts = %{b('scripts')}
        b_www = %{b('www')}

        mods = %{','.join(i.mod_list)}

        rustc = %{i.rustc}
        cc = %{i.cc}
        cxx = %{i.cxx}
        python3 = %{i.python3}
        closure_compiler = %{i.closure_compiler}
        yui_compressor = %{i.yui_compressor}

        user_cflags = %{i.cflags}
        user_cxxflags = %{i.cxxflags}
        user_ldflags = %{i.ldflags}

        version = dev
        site_config = %{i.site_config_path}
    ''', os=os, **locals())


if __name__ == '__main__':
    parser = build_parser()
    args = parser.parse_args(sys.argv[1:])

    log = open('config.log', 'w')
    log.write('Arguments: %r\n\n' % (sys.argv[1:],))

    i, ok = checks.run(args, log)
    if not ok:
        if i.force:
            print('Ignoring errors due to --force')
        else:
            sys.exit(1)

    if i.python3_config is not None:
        py_includes = subprocess.check_output((i.python3_config, '--includes')).decode().strip()
        py_ldflags = subprocess.check_output((i.python3_config, '--ldflags')).decode().strip()
    else:
        py_includes = None
        py_ldflags = None

    if i.debug:
        dist_manifest_base = 'debug.manifest'
    else:
        dist_manifest_base = 'release.manifest'

    dist_manifest = os.path.join(i.root_dir, 'mk', dist_manifest_base)
    common_manifest = os.path.join(i.root_dir, 'mk', 'common.manifest')

    content = header(i)
    content += '\n\n'.join((
        '',
        '# Dist',
        dist.rules(i),
        '',

        '# Server - backend',
        native.rules(i),
        native.rust('syntax_exts', 'dylib', (), extra_flags='-C prefer-dynamic'),
        native.rust('physics', 'lib', ()),
        native.rust('common_types', 'lib', ()),
        native.rust('common_util', 'lib', ('physics', 'common_types',)),
        native.rust('common_proto', 'lib', ('physics', 'common_types')),
        native.rust('common_movement', 'lib', ('physics',)),
        native.rust('common', 'lib', ()),
        native.rust('server_types', 'lib', ('physics', 'common_types', 'common_util')),
        native.rust('server_config', 'lib', ('server_types',)),
        native.rust('server_extra', 'lib', ('server_types',)),
        native.rust('server_world_types', 'lib', ('server_types',)),
        native.rust('python', 'lib',
            ('physics', 'server_types', 'server_config',
                'server_extra', 'common_util', 'server_world_types'),
            dyn_deps=('syntax_exts',)),
        native.rust('server_bundle', 'lib',
            ('physics', 'server_config', 'server_extra', 'server_types',
                'common_util', 'server_world_types')),
        native.rust('server_bundle', 'staticlib',
            ('physics', 'server_config', 'server_extra', 'server_types',
                'common_util', 'server_world_types'),
            extra_flags='--cfg ffi'),
        native.rust('terrain_gen_algo', 'lib', ('server_types',), build_type='release'),
        native.rust('terrain_gen', 'lib',
            ('physics', 'server_config', 'server_types', 'common_util', 'terrain_gen_algo'),
            # Slow terrain gen algorithms cause serious problems in debug
            # builds (3000+ ms to generate each chunk).
            build_type='release'),
        native.rust('backend', 'bin',
            ('physics', 'terrain_gen',
                'common', 'common_movement', 'python',
                'server_bundle', 'server_config', 'server_extra',
                'server_types', 'common_util', 'server_world_types',),
            dyn_deps=('syntax_exts',),
            src_file='$root/src/server/main.rs'),
        native.rust('generate_terrain', 'bin',
            ('physics', 'terrain_gen',
                'server_bundle', 'server_config', 'server_extra', 'server_types',
                'common_util', 'server_world_types')),
        '',

        '# Server - wrapper',
        native.cxx('wrapper', 'bin',
            ('$root/src/wrapper/%s' % f
                for f in os.listdir(os.path.join(i.root_dir, 'src', 'wrapper'))
                if f.endswith('.cpp')),
            cxxflags='-DWEBSOCKETPP_STRICT_MASKING',
            ldflags='-static',
            # TODO: detect these lib flags
            libs='-lboost_system -lpthread' if not i.win32 else
                '-lboost_system-mt -lpthread -lwsock32 -lws2_32'),
        '',

        '# Python libs',
        native.cxx('outpost_savegame', 'shlib',
            ('$root/util/savegame_py/%s' % f
                for f in os.listdir(os.path.join(i.root_dir, 'util/savegame_py'))
                if f.endswith('.c')),
            cflags=py_includes,
            ldflags=py_ldflags,
            ),

        native.rust('terrain_gen_ffi', 'staticlib',
            ('terrain_gen', 'server_config', 'server_types'),
            src_file='$root/src/test_terrain_gen/ffi.rs'),
        native.cxx('outpost_terrain_gen', 'shlib',
            ('$root/src/test_terrain_gen/py.c',),
            cflags=py_includes,
            ldflags=py_ldflags,
            link_extra=['$b_native/libterrain_gen_ffi$_a'],
            ),

        'build pymodules: phony '
            '$b_native/outpost_savegame$_so '
            '$b_native/outpost_terrain_gen$_so',
        '',

        '# Equipment sprite generator',
        native.rust('equip_sprites_render', 'dylib',
                ('physics',),
                src_file='$root/src/gen/equip_sprites/render.rs'),
        dist.copy('$b_native/libequip_sprites_render$_so',
                  '$b_native/equip_sprites_render$_so'),
        '',

        '# Client - asmlibs',
        asmjs.rules(i),
        asmjs.rlib('core', (), i.rust_libcore_src),
        asmjs.rlib('alloc', ('core',), i.rust_liballoc_src),
        asmjs.rlib('rustc_unicode', ('core',), i.rust_librustc_unicode_src),
        asmjs.rlib('collections', ('core', 'alloc', 'rustc_unicode'),
                i.rust_libcollections_src),
        asmjs.rlib('asmrt', ('core',)),
        asmjs.rlib('asmmalloc', ('core', 'asmrt')),
        asmjs.rlib('fakestd', ('core', 'alloc', 'rustc_unicode', 'collections',
            'asmrt', 'asmmalloc')),
        asmjs.rlib('bitflags', ('core',), i.rust_libbitflags_src),
        asmjs.rlib('physics', ('fakestd', 'bitflags')),
        asmjs.rlib('common_types', ('fakestd',)),
        asmjs.rlib('common_util', ('fakestd', 'physics', 'common_types')),
        asmjs.rlib('common_proto', ('fakestd', 'common_types', 'common_util')),
        asmjs.rlib('common_movement', ('fakestd', 'physics')),
        asmjs.rlib('common', ('fakestd',)),
        asmjs.rlib('client_ui_atlas', ('core', 'physics'), '$b_data/ui_atlas.rs'),
        asmjs.rlib('client_fonts', ('core',), '$b_data/fonts_metrics.rs'),
        asmjs.rlib('client', ('fakestd', 'physics',
            'common', 'common_movement', 'common_proto', 'common_types', 'common_util',
            'client_ui_atlas', 'client_fonts')),
        asmjs.asmlibs('asmlibs',
            '$root/src/asmlibs/lib.rs',
            ('core', 'collections', 'asmrt', 'asmmalloc', 'physics', 'client'),
            '$root/src/asmlibs/exports.txt',
            '$root/src/asmlibs/template.js'),
        '',

        '# Client - Javascript',
        js.rules(i),
        js.compile(i, '$b_js/outpost.js', '$root/src/client/js/main.js'),
        js.minify(i, '$b_js/asmlibs.js', '$b_asmjs/asmlibs.js'),
        js.compile(i, '$b_js/configedit.js', '$root/src/client/js/configedit.js'),
        js.client_manifest(i, '$b_js/manifest.json'),
        '',

        '# uvedit',
        asmjs.asmlibs('uvedit_asm',
            '$root/src/uvedit/lib.rs',
            ('core', 'collections', 'asmrt', 'asmmalloc', 'physics'),
            '$root/src/uvedit/asm_exports.txt',
            '$root/src/uvedit/asm_template.js'),

        js.minify(i, '$b_js/uvedit_asm.js', '$b_asmjs/uvedit_asm.js'),
        js.compile(i, '$b_js/uvedit.js', '$root/src/uvedit/main.js'),
        '',

        '# Data',
        data.rules(i),

        data.day_night('$b_data/day_night.json', '$root/assets/misc/day_night_pixels.png'),
        data.font('name', '$root/assets/misc/NeoSans.png'),
        data.font('default_gray', '$root/assets/misc/NeoSans.png',
                extra_args='--color=0x757161'),
        data.font('bold', '$root/assets/misc/NeoSans.png',
            extra_args='--bold'),
        data.font('title', '$root/assets/misc/Alagard.png',
            extra_args='--no-shadow --color=0xdeeed6'),
        data.font('hotbar', '$root/assets/misc/hotbar-font.png',
            charset_args='--char-list="0123456789.k/"'),
        data.font_stack('$b_data/fonts',
                ('name', 'default_gray', 'bold', 'hotbar', 'title')),
        data.server_json('$b_data/server.json'),
        data.ui_atlas('$b_data', '$root/assets/ui_gl/png'),

        data.process(),
        data.binary_defs('$b_data/client_data.bin'),
        data.pack(),

        data.credits('$b_data/credits.html'),
        '',

        '# Server-side scripts',
        scripts.rules(i),
        scripts.copy_mod_scripts(i.mod_list),
        '',

        '# Launcher',
        www.rules(i),

        www.render_template('$b_www/index.html', '$root/src/www/index.html'),
        www.render_template('$b_www/main.css', '$root/src/www/main.css'),
        www.render_template('$b_www/serverlist.html', '$root/src/launcher/serverlist.html'),
        www.render_template('$b_www/launcher.html', '$root/src/launcher/launcher.html'),

        www.render_template('$b_www/templates/_base.html',
                '$root/src/auth/server/templates/_base.html'),

        www.collect_img_lists('$b_www/img/all.txt',
                ('index.html', 'main.css', 'serverlist.html', 'launcher.html',
                    'templates/_base.html')),
        '',

        '# Distribution',
        # dist rules go at the top so other parts can refer to `dist.copy`
        dist.components(i, i.components),
        '',

        'default $builddir/dist.stamp',
        '', # ensure there's a newline after the last command
        ))

    with open('build.ninja', 'w') as f:
        f.write(content)

    print('Generated build.ninja')
    print('Run `ninja` to build')

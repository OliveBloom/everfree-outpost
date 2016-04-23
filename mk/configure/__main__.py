import argparse
import os
import subprocess
import sys

from configure import checks
from configure.gen import native, asmjs, data, js, dist, scripts
from configure.template import template


def build_parser():
    args = argparse.ArgumentParser()

    args.add_argument('--build-dir', default=None,
            help='directory to store build files')
    args.add_argument('--dist-dir', default=None,
            help='directory to store distribution image')

    args.add_argument('--data-only', action='store_true', default=False,
            help='generate data files only; don\'t compile any code')
    args.add_argument('--use-prebuilt',
            help='use prebuild versions of the named files/directories')
    args.add_argument('--prebuilt-dir', default=None,
            help='directory containing a previously compiled version')
    args.add_argument('--reconfigure', action='store_true', default=False,
            help='reuse cached configuration info when possible')
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

    args.add_argument('--with-server-gui', action='store_true',
            help='include server_gui.py in the build')

    return args


class Info(object):
    def __init__(self, args):
        self._args = args

        script_dir = os.path.dirname(sys.argv[0])
        if script_dir == '':
            self.root_dir = '.'
        else:
            self.root_dir = os.path.normpath(os.path.join(script_dir, '..', '..'))

        in_tree = self.root_dir == '.' or self.root_dir == os.getcwd()

        if args.build_dir is None:
            self.build_dir = 'build' if in_tree else '.'
        else:
            self.build_dir = args.build_dir

        if args.dist_dir is None:
            self.dist_dir = 'dist' if in_tree else os.path.join(self.build_dir, 'dist')
        else:
            self.dist_dir = args.dist_dir

    def __getattr__(self, k):
        return getattr(self._args, k)


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
        prebuilt = %{os.path.normpath(i.prebuilt_dir or '')}

        _exe = %{'' if not i.win32 else '.exe'}
        _so = %{'.so' if not i.win32 else '.dll'}
        _a = .a

        b_native = %{b('native')}
        b_asmjs = %{b('asmjs')}
        b_data = %{b('data')}
        b_js = %{b('js')}
        b_scripts = %{b('scripts')}

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
    ''', os=os, **locals())


def fix_bitflags(src_out, src_in):
    return template('''
        rule fix_bitflags_src
            command = $
                echo '#![feature(no_std)]' >$out && $
                echo '#![no_std]' >>$out && $
                cat $in >> $out
            description = PATCH bitflags.rs

        build %src_out: fix_bitflags_src %src_in
    ''', **locals())


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
    maybe_data_filter = os.path.join(i.root_dir, 'mk', 'data_files.txt') \
            if i.data_only else None

    dist_extra = []
    if i.with_server_gui:
        dist_extra.append(('server_gui.py', '$root/util/server_gui.py'))

    content = header(i)

    if not i.data_only:
        content += '\n\n'.join((
            '',
            '# Native',
            native.rules(i),
            native.rust('syntax_exts', 'dylib', (), extra_flags='-C prefer-dynamic'),
            native.rust('physics', 'lib', ()),
            native.rust('server_types', 'lib', ('physics',)),
            native.rust('server_config', 'lib', ('server_types',)),
            native.rust('server_extra', 'lib', ('server_types',)),
            native.rust('server_util', 'lib', ('server_types',)),
            native.rust('server_world_types', 'lib', ('server_types',)),
            native.rust('server_bundle', 'lib',
                ('physics', 'server_config', 'server_extra', 'server_types',
                    'server_util', 'server_world_types')),
            native.rust('server_bundle', 'staticlib',
                ('physics', 'server_config', 'server_extra', 'server_types',
                    'server_util', 'server_world_types'),
                extra_flags='--cfg ffi'),
            native.rust('terrain_gen_algo', 'lib', ('server_types',), build_type='release'),
            native.rust('terrain_gen', 'lib',
                ('physics', 'server_config', 'server_types', 'server_util', 'terrain_gen_algo'),
                # Slow terrain gen algorithms cause serious problems in debug
                # builds (3000+ ms to generate each chunk).
                build_type='release'),
            native.rust('backend', 'bin',
                ('physics', 'terrain_gen',
                    'server_bundle', 'server_config', 'server_extra',
                    'server_types', 'server_util', 'server_world_types',),
                dyn_deps=('syntax_exts',),
                src_file='$root/src/server/main.rs'),
            native.cxx('wrapper', 'bin',
                ('$root/src/wrapper/%s' % f
                    for f in os.listdir(os.path.join(i.root_dir, 'src', 'wrapper'))
                    if f.endswith('.cpp')),
                cxxflags='-DWEBSOCKETPP_STRICT_MASKING',
                ldflags='-static',
                # TODO: detect these lib flags
                libs='-lboost_system -lpthread' if not i.win32 else
                    '-lboost_system-mt -lpthread -lwsock32 -lws2_32'),
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

            '# Asm.js',
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
            asmjs.rlib('client_ui_atlas', ('physics',), '$b_data/ui_atlas.rs'),
            asmjs.rlib('client_fonts', (), '$b_data/fonts_metrics.rs'),
            asmjs.rlib('client', ('fakestd', 'physics',
                'client_ui_atlas', 'client_fonts')),
            asmjs.asmlibs('asmlibs',
                '$root/src/asmlibs/lib.rs',
                ('core', 'collections', 'asmrt', 'asmmalloc', 'physics', 'client'),
                '$root/src/asmlibs/exports.txt',
                '$root/src/asmlibs/template.js'),

            '# Javascript',
            js.rules(i),
            js.compile(i, '$b_js/outpost.js', '$root/src/client/js/main.js'),
            js.minify('$b_js/asmlibs.js', '$b_asmjs/asmlibs.js'),
            js.compile(i, '$b_js/animtest.js', '$root/src/client/js/animtest.js'),
            js.compile(i, '$b_js/configedit.js', '$root/src/client/js/configedit.js'),
            ))

    content += '\n\n'.join((
        '',
        '# Data',
        data.rules(i),
        data.font('name', '$root/assets/misc/NeoSans.png'),
        data.font('title', '$root/assets/misc/Alagard.png',
            extra_args='--no-shadow --color=0xdeeed6'),
        data.font('hotbar', '$root/assets/misc/hotbar-font.png',
            charset_args='--char-list="0123456789.k"'),
        data.font_stack('$b_data/fonts', ('name', 'hotbar', 'title')),
        data.day_night('$b_data/day_night.json', '$root/assets/misc/day_night_pixels.png'),
        data.server_json('$b_data/server.json'),
        data.ui_atlas('$b_data', '$root/assets/ui_gl/png'),
        data.process(),
        data.binary_defs('$b_data/client_data.bin'),
        data.pack(),
        data.credits('$b_data/credits.html'),

        '# Server-side scripts',
        scripts.rules(i),
        scripts.copy_mod_scripts(i.mod_list),

        '# Distribution',
        dist.rules(i),
        dist.from_manifest(common_manifest, dist_manifest,
                filter_path=maybe_data_filter,
                exclude_names=i.use_prebuilt,
                extra=dist_extra),

        'default $builddir/dist.stamp',
        '', # ensure there's a newline after the last command
        ))

    with open('build.ninja', 'w') as f:
        f.write(content)

    print('Generated build.ninja')
    print('Run `ninja` to build')

import argparse
import os
import subprocess
import sys

from configure import checks
from configure.gen import native, asmjs, data, js, dist
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

    args.add_argument('--debug', action='store_true',
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
    args.add_argument('--emscripten-passes-prefix', default=None,
            help='path to epdtry/rust-emscripten-passes build directory')

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

    args.add_argument('--force', action='store_true',
            help='proceed even if there are configuration errors')


    return args


class Info(object):
    def __init__(self, args):
        self._args = args

        script_dir = os.path.dirname(sys.argv[0])
        if script_dir == '':
            self.src_dir = '.'
        else:
            self.src_dir = os.path.normpath(os.path.join(script_dir, '..', '..'))

        in_tree = self.src_dir == '.' or self.src_dir == os.getcwd()

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
        src = %{os.path.normpath(i.src_dir)}
        # Note: (1) `build` is a ninja keyword; (2) `builddir` is a special
        # variable that determines where `.ninja_log` is stored.
        builddir = %{os.path.normpath(i.build_dir)}
        dist = %{os.path.normpath(i.dist_dir)}
        prebuilt = %{os.path.normpath(i.prebuilt_dir or '')}

        b_native = %{b('native')}
        b_asmjs = %{b('asmjs')}
        b_data = %{b('data')}
        b_js = %{b('js')}

        rust_home = %{i.rust_home}
        bitflags_home = %{i.bitflags_home}

        rustc = %{i.rustc}
        cc = %{i.cc}
        cxx = %{i.cxx}
        python3 = %{i.python3}
        closure_compiler = closure-compiler
        yui_compressor = yui-compressor
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
    i = Info(args)

    log = open('config.log', 'w')
    log.write('Arguments: %r\n\n' % (sys.argv[1:],))

    if not checks.run(i, log):
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

    dist_manifest = os.path.join(i.src_dir, 'mk', dist_manifest_base)
    common_manifest = os.path.join(i.src_dir, 'mk', 'common.manifest')
    maybe_data_filter = os.path.join(i.src_dir, 'mk', 'data_files.txt') \
            if i.data_only else None

    content ='\n\n'.join((
        header(i),

        '# Native',
        native.rules(i),
        native.rust('physics', 'lib', ()),
        native.rust('backend', 'bin', ('physics',), '$src/server/main.rs'),
        native.cxx('wrapper', 'bin',
            ('$src/wrapper/%s' % f for f in os.listdir(os.path.join(i.src_dir, 'wrapper'))
                if f.endswith('.cpp')),
            cxxflags='-DWEBSOCKETPP_STRICT_MASKING',
            ldflags='-static',
            libs='-lboost_system -lpthread'),
        native.cxx('outpost_savegame', 'shlib',
            ('$src/util/savegame_py/%s' % f
                for f in os.listdir(os.path.join(i.src_dir, 'util/savegame_py'))
                if f.endswith('.c')),
            cflags=py_includes,
            ldflags=py_ldflags,
            ),

        '# Asm.js',
        asmjs.rules(i),
        asmjs.rlib('core', (), '$rust_home/src/libcore/lib.rs'),
        fix_bitflags('$b_asmjs/bitflags.rs', '$bitflags_home/src/lib.rs'),
        asmjs.rlib('bitflags', (), '$b_asmjs/bitflags.rs'),
        asmjs.rlib('asmrt', ('core',)),
        asmjs.rlib('physics', ('core', 'bitflags', 'asmrt')),
        asmjs.rlib('graphics', ('core', 'asmrt', 'physics')),
        asmjs.asmlibs('asmlibs',
            '$src/client/asmlibs.rs', ('core', 'asmrt', 'physics', 'graphics'),
            '$src/client/asmlibs_exports.txt', '$src/client/asmlibs.tmpl.js'),

        '# Data',
        data.rules(i),
        data.font('$b_data/font', '$src/assets/misc/NeoSans.png'),
        data.day_night('$b_data/day_night.json', '$src/assets/misc/day_night_pixels.png'),
        data.server_json('$b_data/server.json'),
        data.process(),
        data.pack(),
        data.credits('$b_data/credits.html'),

        '# Javascript',
        js.rules(i),
        js.compile(i, '$b_js/outpost.js', '$src/client/js/main.js'),
        js.minify('$b_js/asmlibs.js', '$b_asmjs/asmlibs.js'),
        js.compile(i, '$b_js/animtest.js', '$src/client/js/animtest.js'),
        js.compile(i, '$b_js/configedit.js', '$src/client/js/configedit.js'),

        '# Distribution',
        dist.rules(i),
        dist.from_manifest(common_manifest, dist_manifest,
                filter_path=maybe_data_filter,
                exclude_names=i.use_prebuilt),

        'default $builddir/dist.stamp',
        '', # ensure there's a newline after the last command
        ))

    with open('build.ninja', 'w') as f:
        f.write(content)

    print('Generated build.ninja')
    print('Run `ninja` to build')

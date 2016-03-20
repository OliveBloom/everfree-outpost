import os
import platform
import sys
import tempfile

def pre_configure(ctx):
    '''Compute basic info.  This step avoids referring to ctx.args as much as
    possible, since that introduces dependencies that can prevent --reconfigure
    from being effective.'''
    p = platform.system()
    ctx.out('Checking platform: %s' % p)
    ctx.info.add('win32', 'platform is win32')
    ctx.info.win32 = (p == 'Windows')

    ctx.info.add('root_dir', 'source directory')
    script_dir = os.path.dirname(sys.argv[0])
    if script_dir == '':
        ctx.info.root_dir = '.'
    else:
        ctx.info.root_dir = os.path.normpath(os.path.join(script_dir, '..', '..'))

    ctx.info.add('in_tree', 'build is in-tree')
    ctx.info.in_tree = ctx.info.root_dir == '.' or ctx.info.root_dir == os.getcwd()

    ctx.info.add('build_dir', 'build directory')
    # args.build_dir is fine.  The cache used by --reconfigure is stored there,
    # so the same cache always has the same build_dir.
    if ctx.args.build_dir is None:
        ctx.info.build_dir = 'build' if ctx.info.in_tree else '.'
    else:
        ctx.info.build_dir = ctx.args.build_dir

def post_configure(ctx):
    ctx.info.add('dist_dir', 'distribution directory')
    if ctx.args.dist_dir is None:
        ctx.info.dist_dir = 'dist' if ctx.info.in_tree \
                else os.path.join(ctx.info.build_dir, 'dist')
    else:
        ctx.info.dist_dir = ctx.args.dist_dir

    ctx.info.add('debug', 'debug build')
    ctx.info.debug = ctx.args.debug

    ctx.info.add('with_server_gui', 'include server_gui.py')
    ctx.info.with_server_gui = ctx.args.with_server_gui

    ctx.info.add('mod_list', 'included mods')
    ctx.info.mod_list = ('outpost',) + \
            (tuple(ctx.args.mods.split(',')) if ctx.args.mods else ())


    ctx.info.add('data_only', 'build data only?')
    ctx.info.data_only = ctx.args.data_only

    ctx.info.add('use_prebuilt', 'use prebuilt files')
    ctx.info.use_prebuilt = ctx.args.use_prebuilt

    ctx.info.add('prebuilt_dir', 'path to prebuilt files')
    ctx.info.prebuilt_dir = ctx.args.prebuilt_dir

def check(ctx, need_vars):
    ok = True

    if ctx.info.data_only and ctx.info.prebuilt_dir is None:
        ctx.out('Error: --prebuilt-dir must be set because --data-only is set',
                level='ERR')
        ok = False
    if ctx.info.use_prebuilt and ctx.info.prebuilt_dir is None:
        ctx.out('Error: --prebuilt-dir must be set because --use-prebuilt is set',
                level='ERR')
        ok = False

    for k in need_vars:
        if getattr(ctx.info, k) is None:
            ctx.out('Error: Failed to detect %s' % ctx.info._descs[k])
            ok = False

    return ok

def run(args, log_file):
    from . import context, cc, rustc, python, emscripten, js

    with tempfile.TemporaryDirectory() as temp_dir:
        ctx = context.Context(args, temp_dir, log_file)

        pre_configure(ctx)

        if not ctx.load_cache():
            cc.configure(ctx)
            rustc.configure(ctx)
            python.configure(ctx)
            emscripten.configure(ctx)
            js.configure(ctx)

        ctx.save_cache()

        post_configure(ctx)

        print('')

        reqs = []
        reqs.extend(cc.requirements(ctx))
        reqs.extend(rustc.requirements(ctx))
        reqs.extend(python.requirements(ctx))
        reqs.extend(emscripten.requirements(ctx))
        reqs.extend(js.requirements(ctx))
        ok = check(ctx, reqs)

        ctx.log('Configuration settings:')
        for k,v in sorted(ctx.info._values.items()):
            ctx.log('  %-20s %s' % (k + ':', v))

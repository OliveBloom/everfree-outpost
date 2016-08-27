import os
import platform
import sys
import tempfile

COMPONENT_ALIASES = {
        'server': ('server-code', 'server-data'),
        'client': ('client-code', 'client-data'),
        'all': (
            'server',
            'client',
            'launcher',
            'auth',
            'uvedit',
            'website',
            )
        }

def parse_components(s):
    parts = s.split(',')
    cs = set()

    def go(p):
        if p in COMPONENT_ALIASES:
            for q in COMPONENT_ALIASES[p]:
                go(q)
            return

        cs.add(p)

    for p in parts:
        go(p)

    return tuple(sorted(cs))


def pre_configure(ctx):
    '''Compute basic info.  This step avoids referring to ctx.args as much as
    possible, since that introduces dependencies that can prevent --reconfigure
    from being effective.'''
    p = platform.system()
    ctx.out('Checking platform: %s' % p)
    ctx.info.add('win32', 'platform is win32')
    ctx.info.win32 = (p == 'Windows')

    ctx.info.add('root_dir', 'source directory')
    if ctx.args.root_dir is None:
        script_dir = os.path.dirname(sys.argv[0])
        ctx.info.root_dir = os.path.normpath(os.path.join(script_dir, '..', '..'))
    else:
        ctx.info.root_dir = ctx.args.root_dir

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

    ctx.copy_arg('debug', 'debug build')

    ctx.info.add('mod_list', 'included mods')
    ctx.info.mod_list = ('outpost',) + \
            (tuple(ctx.args.mods.split(',')) if ctx.args.mods else ())

    ctx.info.add('components', 'components to build')
    ctx.info.components = parse_components(ctx.args.components)

    ctx.info.add('site_config_path', 'site config file')
    if ctx.args.site_config is None:
        ctx.info.site_config_path = '$root/util/site.yaml'
    else:
        ctx.info.site_config_path = os.path.abspath(ctx.args.site_config)

    ctx.copy_arg('cflags', 'extra C compiler flags', default='')
    ctx.copy_arg('cxxflags', 'extra C++ compiler flags', default='')
    ctx.copy_arg('ldflags', 'extra C/C++ linker flags', default='')

    ctx.copy_arg('force', 'ignore configuration errors')

def check(ctx, need_vars):
    ok = True

    for k in need_vars:
        if getattr(ctx.info, k) is None:
            ctx.out('Error: Failed to detect %s' % ctx.info._descs[k])
            ok = False

    return ok

def run(args, log_file):
    from . import context, cc, rustc, python, emscripten, js, misc

    with tempfile.TemporaryDirectory() as temp_dir:
        ctx = context.Context(args, temp_dir, log_file)

        pre_configure(ctx)

        if not ctx.load_cache():
            cc.configure(ctx)
            rustc.configure(ctx)
            python.configure(ctx)
            emscripten.configure(ctx)
            js.configure(ctx)
            misc.configure(ctx)

        ctx.save_cache()

        post_configure(ctx)

        print('')

        reqs = []
        reqs.extend(cc.requirements(ctx))
        reqs.extend(rustc.requirements(ctx))
        reqs.extend(python.requirements(ctx))
        reqs.extend(emscripten.requirements(ctx))
        reqs.extend(js.requirements(ctx))
        reqs.extend(misc.requirements(ctx))
        ok = check(ctx, reqs)

        ctx.out('Configuration settings:')
        for k,v in sorted(ctx.info._values.items()):
            ctx.out('  %-40s %s' % (ctx.info._descs[k] + ':', v))
        print('')

        return ctx.info, ok

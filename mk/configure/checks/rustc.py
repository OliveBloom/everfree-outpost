import os

from configure.checks.context import ConfigError

NEED_RUSTC_HASH = 'a5d1e7a59'

NEED_RUST_LIBS = (
    # Keep these in dependency order.  That way necessary --extern flags will already be
    # set before they are needed for later checks.
    'libc',
    'bitflags',
    'rand',
    'memchr',
    'aho_corasick',
    'utf8_ranges',
    'regex_syntax',
    'regex',
    'log',
    'env_logger',
    'rustc_serialize',
    'time',
    'python3_sys',
    'libsqlite3_sys',
    'rusqlite',
    'linked_hash_map',
    'lru_cache',
    'vec_map',
    )

NEED_RUST_LIB_SRC = (
        'core',
        'alloc',
        'rustc_unicode',
        'collections',
        'bitflags',
        # If you extend this, also add a new check in configure()
        )

def configure(ctx):
    ctx.detect('rustc', 'Rust compiler', ('rustc',), chk_rustc)

    ctx.info.add('rust_externs', 'Rust library --extern flags')
    ext_arg = ctx.args.rust_lib_externs
    ctx.info.rust_externs = ext_arg.split(',') if ext_arg else []

    for lib in NEED_RUST_LIBS:
        configure_lib(ctx, lib)

    configure_lib_src(ctx, 'core', ctx.args.rust_home, 'src/libcore')
    configure_lib_src(ctx, 'alloc', ctx.args.rust_home, 'src/liballoc')
    configure_lib_src(ctx, 'rustc_unicode', ctx.args.rust_home, 'src/librustc_unicode')
    configure_lib_src(ctx, 'collections', ctx.args.rust_home, 'src/libcollections')
    configure_lib_src(ctx, 'bitflags', ctx.args.bitflags_home)

    ctx.copy_arg('rust_extra_libdir', 'Rust extra library directory')

def requirements(ctx):
    return ('rustc',) + \
            tuple('rust_lib%s_path' % l for l in NEED_RUST_LIBS) + \
            tuple('rust_lib%s_src' % l for l in NEED_RUST_LIB_SRC)

def configure_lib(ctx, crate_name):
    key = 'rust_lib%s_path' % crate_name
    desc = 'Rust lib%s library path' % crate_name
    lib_desc = 'Rust lib%s library' % crate_name
    ctx.info.add(key, desc)

    if ctx.info.rustc is None:
        ctx.out_skip(key, 'rustc')
        return

    # Set up for the tests
    src = ctx.write('rs', 'extern crate %s; fn main() {}' % crate_name)
    out = ctx.file('exe')

    lib_dir_flags = ()
    extern = None
    if ctx.args.rust_extra_libdir is not None:
        lib_dir_flags = ('-L', ctx.args.rust_extra_libdir)
        extern = os.path.join(ctx.args.rust_extra_libdir, 'lib%s.rlib' % crate_name)

    extern_flag = ('--extern', '%s=%s' % (crate_name, extern),)

    other_extern_flags = ()
    for e in ctx.info.rust_externs:
        other_extern_flags += ('--extern', e)


    path = getattr(ctx.args, key, None)

    if path is None:
        ctx.out_part('Checking for %s: ' % lib_desc)
        if ctx.run(ctx.info.rustc,
                lib_dir_flags + other_extern_flags + (src, '-o', out)):
            ctx.out('ok')
            path = ''
        else:
            ctx.out('not found')

    if path is None and extern is not None:
        ctx.out_part('Checking for %s (with --extern): ' % lib_desc)
        if ctx.run(ctx.info.rustc,
                lib_dir_flags + other_extern_flags + extern_flag + (src, '-o', out)):
            ctx.out('ok')
            path = extern
        else:
            ctx.out('not found')

    setattr(ctx.info, key, path)
    if path is not None and path != '':
        ctx.info.rust_externs.append('%s=%s' % (crate_name, extern))

def configure_lib_src(ctx, crate_name, home, rel_dir='src'):
    candidates = (os.path.join(home, rel_dir, 'lib.rs'),)

    def chk(ctx, path):
        if not os.path.isfile(path):
            raise ConfigError('not found')
        return True

    key = 'rust_lib%s_src' % crate_name
    desc = 'Rust lib%s source file' % crate_name
    ctx.detect(key, desc, candidates, chk)


def chk_rustc(ctx, rustc):
    out = ctx.run_output(rustc, ('--version',))
    if out is None:
        raise ConfigError('not found')

    ver = out.splitlines()[0]
    if NEED_RUSTC_HASH not in ver:
        raise ConfigError('bad version %r (need %r)' % (ver.strip(), NEED_RUSTC_HASH))

    return True

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

def configure(ctx):
    ctx.detect('rustc', 'Rust compiler', ('rustc',), chk_rustc)

    ctx.info.add('rust_externs', 'Rust library --extern flags')
    ctx.info.rust_externs = []

    for lib in NEED_RUST_LIBS:
        configure_lib(ctx, lib)

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


def chk_rustc(ctx, rustc):
    out = ctx.run_output(rustc, ('--version',))
    if out is None:
        raise ConfigError('not found')

    ver = out.splitlines()[0]
    if NEED_RUSTC_HASH not in ver:
        raise ConfigError('bad version %r (need %r)' % (ver.strip(), NEED_RUSTC_HASH))

    return True

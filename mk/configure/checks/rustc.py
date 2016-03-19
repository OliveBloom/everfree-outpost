from configure.checks.context import ConfigError

NEED_RUSTC_HASH = 'a5d1e7a59'

def configure(ctx):
    ctx.detect('rustc', 'Rust compiler', ('rustc',), chk_rustc)

def chk_rustc(ctx, rustc):
    out = ctx.run_output(rustc, ('--version',))
    if out is None:
        raise ConfigError('not found')

    ver = out.splitlines()[0]
    if NEED_RUSTC_HASH not in ver:
        raise ConfigError('bad version %r (need %r)' % (ver.strip(), NEED_RUSTC_HASH))

    return True

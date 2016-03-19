from configure.checks.context import ConfigError

NEED_RUSTC_HASH = 'dc6e3bbb7'

def find_rustc(ctx):
    candidates = ctx.get_candidates('rustc', ('rustc',))

    def chk(ctx, rustc):
        out = ctx.run_output(rustc, ('--version',))
        if out is None:
            raise ConfigError('not found')

        ver = out.splitlines()[0]
        if NEED_RUSTC_HASH not in ver:
            raise ConfigError('bad version %r (need %r)' % (ver.strip(), NEED_RUSTC_HASH))

        return True

    ctx.info.rustc = ctx.check_all('Rust compiler', candidates, chk)

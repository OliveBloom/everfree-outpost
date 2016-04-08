import os

from configure.checks.context import ConfigError

def configure(ctx):
    ctx.detect('emscripten_fastcomp_prefix', 'emscripten-fastcomp installation',
            ('', '/usr', '/usr/local'), chk_fastcomp)

def requirements(ctx):
    if ctx.info.data_only:
        return ()
    else:
        return ('emscripten_fastcomp_prefix',)


def chk_fastcomp(ctx, prefix):
    if prefix == '':
        llc = 'llc'
    else:
        llc = os.path.join(prefix, 'bin', 'llc')

    if not ctx.run(llc, ('-version',)):
        raise ConfigError('not found')
    if not ctx.run(llc, ('-march=js', '-')):
        raise ConfigError('missing JS backend support')
    return True

def chk_plugins(ctx, prefix):
    opt = os.path.join(ctx.info.emscripten_fastcomp_prefix, 'bin', 'opt')

    def check(shlib, flag):
        shlib_path = os.path.join(prefix, shlib)
        # `opt` version 3.4 returns 1 on -help/-version for some reason.
        output = ctx.run_output(opt, ['-load', shlib_path, '-help'])
        if flag not in output:
            raise ConfigError('failed to load plugin %s' % shlib)

    check('RemoveOverflowChecks.so', '-remove-overflow-checks')
    check('RemoveAssume.so', '-remove-assume')
    return True

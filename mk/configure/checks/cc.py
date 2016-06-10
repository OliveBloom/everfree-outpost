from configure.checks.context import ConfigError

def configure(ctx):
    out = ctx.file('exe')

    src = ctx.write('c', 'int main() { return 37; }')
    ctx.detect('cc', 'C compiler', ('cc', 'gcc', 'clang'),
            lambda ctx, cxx: chk_cc(ctx, cxx, src, out))

    src = ctx.write('cpp', 'int main() { return 37; }')
    ctx.detect('cxx', 'C++ compiler', ('c++', 'g++', 'clang++'),
            lambda ctx, cxx: chk_cc(ctx, cxx, src, out))

def requirements(ctx):
    return ('cc', 'cxx')


def chk_cc(ctx, cc, src, out):
    if not ctx.run(cc, (src, '-o', out)):
        raise ConfigError('not found')
    if not ctx.run(out, expect_ret=37):
        raise ConfigError('error testing compiled program')
    return True

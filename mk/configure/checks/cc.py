from configure.checks.context import ConfigError

def find_cc(ctx):
    candidates = ctx.get_candidates('cc', ('cc', 'gcc', 'clang'))

    src = ctx.write('c', 'int main() { return 37; }')
    out = ctx.file('exe')

    def chk(ctx, cc):
        if not ctx.run(cc, (src, '-o', out)):
            raise ConfigError('not found')
        if not ctx.run(out, expect_ret=37):
            raise ConfigError('error testing compiled program')
        return True

    ctx.info.cc = ctx.check_all('C compiler', candidates, chk)

def find_cxx(ctx):
    candidates = ctx.get_candidates('cxx', ('c++', 'g++', 'clang++'))

    src = ctx.write('cpp', 'int main() { return 37; }')
    out = ctx.file('exe')

    def chk(ctx, cxx):
        if not ctx.run(cxx, (src, '-o', out)):
            raise ConfigError('not found')
        if not ctx.run(out, expect_ret=37):
            raise ConfigError('error testing compiled program')
        return True

    ctx.info.cxx = ctx.check_all('C++ compiler', candidates, chk)

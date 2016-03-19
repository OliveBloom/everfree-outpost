import ast

from configure.checks.context import ConfigError

NEED_PYTHON3_VERSION = (3, 4)

def find_python3(ctx):
    candidates = ctx.get_candidates('python3',
            ('python3', 'python', 'python3.4', 'python3.5', 'python3.6'))

    def chk(ctx, python3):
        out = ctx.run_output(python3, ('-c', 'import sys; print(tuple(sys.version_info))'))
        if out is None:
            raise ConfigError('not found')

        try:
            ver = ast.literal_eval(out)
        except ValueError:
            raise ConfigError('bad output')

        if ver < NEED_PYTHON3_VERSION:
            raise ConfigError('bad version %s (need %s or greater)' %
                    ('.'.join(str(x) for x in ver),
                        '.'.join(str(x) for x in NEED_PYTHON3_VERSION)))

        return True

    ctx.info.python3 = ctx.check_all('Python 3 interpreter', candidates, chk)

def find_python3_config(ctx):
    if ctx.info.python3 is None:
        ctx.out_skip('Python 3 configuration helper', 'Python 3 interpreter')
        return

    candidates = ctx.get_candidates('python3_config', (ctx.info.python3 + '-config'))

    def chk(ctx, python3_config):
        if not ctx.run(python3_config, ('--help',)):
            raise ConfigError('not found')
        return True

    ctx.info.python3_config = ctx.check_all('Python 3 configuration helper', candidates, chk)

def check_python3_lib(ctx, desc, key, candidates):
    if ctx.info.python3 is None:
        ctx.out_skip('Python 3 %s' % desc, 'Python 3 interpreter')
        return

    candidates = getattr(ctx.args, 'python3_%s' % key, None) or candidates

    def chk(ctx, module):
        if not ctx.run(ctx.info.python3, ('-c', 'import %s' % module)):
            raise ConfigError('not found')
        return True

    ok = ctx.check_all('Python 3 %s' % desc, candidates, chk)
    setattr(ctx.info, 'has_python3_%s' % key, bool(ok))

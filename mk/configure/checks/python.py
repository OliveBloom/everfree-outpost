import ast

from configure.checks.context import ConfigError

NEED_PYTHON3_VERSION = (3, 4)

def configure(ctx):
    ctx.detect('python3', 'Python 3 interpreter',
            ('python3', 'python', 'python3.7', 'python3.6', 'python3.5', 'python3.4'),
            chk_python3)

    ctx.info.add('python3_config', 'Python 3 configuration helper')
    if ctx.info.python3 is None:
        ctx.warn_skip('python3_config', 'python3')
    else:
        ctx.detect_('python3_config', (ctx.info.python3 + '-config',),
                chk_python3_config)

    ctx.detect('python3_pil', 'Python 3 imaging library', ('PIL',),
            chk_python3_lib, deps=('python3',))
    ctx.detect('python3_yaml', 'Python 3 YAML library', ('yaml',),
            chk_python3_lib, deps=('python3',))
    ctx.detect('python3_json', 'Python 3 JSON library', ('simplejson', 'json'),
            chk_python3_lib, deps=('python3',))

def requirements(ctx):
    return ('python3', 'python3_pil', 'python3_yaml', 'python3_json', 'python3_config')


def chk_python3(ctx, python3):
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

def chk_python3_config(ctx, python3_config):
    if not ctx.run(python3_config, ('--help',)):
        raise ConfigError('not found')
    return True

def chk_python3_lib(ctx, module):
    if not ctx.run(ctx.info.python3, ('-c', 'import %s' % module)):
        raise ConfigError('not found')
    return True

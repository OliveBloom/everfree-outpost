from configure.checks.context import ConfigError

def configure(ctx):
    out = ctx.file('exe')

    ctx.detect('pandoc', 'Pandoc', ('pandoc',), chk_pandoc)

def requirements(ctx):
    return ('pandoc',)


def chk_pandoc(ctx, pandoc):
    if not ctx.run(pandoc, ('--version',)):
        raise ConfigError('not found')
    return True

import os

from configure.checks.context import ConfigError

def configure(ctx):
    ctx.detect('closure_compiler', 'Closure Compiler',
            ('closure-compiler',), chk_closure)
    ctx.detect('yui_compressor', 'YUI Compressor',
            ('yui-compressor',), chk_yui)

def requirements(ctx):
    if ctx.info.data_only:
        return ()
    else:
        return ('closure_compiler', 'yui_compressor')


def chk_closure(ctx, closure):
    # Strangely, closure-compiler --help seems to return 255, not 0.
    # We could use closure-compiler --version instead, but that takes much
    # longer to run (several seconds).
    if not ctx.run(closure, ('--help',), expect_ret=255):
        raise ConfigError('not found')
    return True

def chk_yui(ctx, yui):
    if not ctx.run(yui, ('--help',)):
        raise ConfigError('not found')
    return True

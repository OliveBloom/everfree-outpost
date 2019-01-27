import os

from configure.checks.context import ConfigError

def configure(ctx):
    ctx.detect('closure_compiler', 'Closure Compiler',
            ('closure-compiler',), chk_closure)
    ctx.detect('yui_compressor', 'YUI Compressor',
            ('yui-compressor', 'yuicompressor'), chk_yui)

def requirements(ctx):
    return ('closure_compiler', 'yui_compressor')


def chk_closure(ctx, closure):
    if not ctx.run(closure, ('--version',)):
        raise ConfigError('not found')
    return True

def chk_yui(ctx, yui):
    if not ctx.run(yui, ('--help',)):
        raise ConfigError('not found')
    return True

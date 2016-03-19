import platform
import tempfile

def basic_configure(ctx):
    p = platform.system()
    ctx.out('Checking platform: %s' % p)
    ctx.info.add('win32', 'platform is win32')
    ctx.info.win32 = (p == 'Windows')

def run(args, log_file):
    from . import context, cc, rustc, python

    with tempfile.TemporaryDirectory() as temp_dir:
        ctx = context.Context(args, temp_dir, log_file)

        basic_configure(ctx)
        cc.configure(ctx)
        rustc.configure(ctx)
        python.configure(ctx)

        print('')
        print('Configuration settings:')
        for k,v in sorted(ctx.info._values.items()):
            print('  %-20s %s' % (k + ':', v))

import tempfile

def run(args, log_file):
    from . import context, cc, rustc, python

    with tempfile.TemporaryDirectory() as temp_dir:
        ctx = context.Context(args, temp_dir, log_file)

        cc.find_cc(ctx)
        cc.find_cxx(ctx)
        rustc.find_rustc(ctx)
        python.find_python3(ctx)
        python.find_python3_config(ctx)
        python.check_python3_lib(ctx, 'image library', 'pil', ('PIL.Image',))
        python.check_python3_lib(ctx, 'YAML library', 'yaml', ('yaml',))
        python.check_python3_lib(ctx, 'JSON library', 'json', ('simplejson', 'json',))

        print('')
        print('Configuration settings:')
        for k,v in sorted(ctx.info.__dict__.items()):
            print('  %-20s %s' % (k + ':', v))

import argparse
import base64
import hashlib
import os
import shutil
import subprocess
import sys
import yaml

class DictObj:
    def __init__(self, dct):
        self._dct = dct

    def __getattr__(self, k):
        return self._dct[k]

def wrap(x):
    if isinstance(x, dict):
        return DictObj({k: wrap(v) for k,v in x.items()})
    if isinstance(x, (list, tuple)):
        return tuple(wrap(y) for y in x)
    return x

def build_parser():
    args = argparse.ArgumentParser()

    args.add_argument('--img-src', default=None,
            help='source directory for referenced images')
    args.add_argument('--img-dir', default=None,
            help='destination directory for optimized images')
    args.add_argument('--optimize', default=None,
            help='optimize images using the indicated optipng binary')
    args.add_argument('--site-config',
            help='path to the site config file')

    args.add_argument('--in-file',
            help='path to input file')
    args.add_argument('--out-file',
            help='path to output file')

    return args

SRC_IMAGES = set()
USED_IMAGES = set()

def get_image(args, src, rel=None):
    src = os.path.join(args.img_src, src)
    SRC_IMAGES.add(src)

    with open(src, 'rb') as f:
        hasher = hashlib.sha256()
        while True:
            b = f.read(4096)
            if len(b) == 0:
                break
            hasher.update(b)
        h = hasher.hexdigest()

    _, ext = os.path.splitext(src)
    name = h[:8] + ext
    dest = os.path.join(args.img_dir, name)

    if not os.path.exists(dest) or os.path.getmtime(dest) < os.path.getmtime(src):
        if args.optimize is None or ext != '.png':
            shutil.copyfile(src, dest)
        else:
            subprocess.check_call((args.optimize, '-out', dest, src))

    if os.path.getsize(dest) < 1024 and args.optimize is not None and ext == '.png':
        with open(dest, 'rb') as f:
            data = f.read()
            b = base64.b64encode(data).decode('ascii')
            url = 'data:image/png;base64,%s' % b
    else:
        USED_IMAGES.add(name)
        url = (rel or '') + name

    return url

if __name__ == '__main__':
    sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
    from configure.template import template

    parser = build_parser()
    args = parser.parse_args(sys.argv[1:])

    os.makedirs(args.img_dir, exist_ok=True)

    with open(args.site_config) as f:
        cfg = yaml.load(f)


    img_base = ['img/']
    def img_url(src):
        return get_image(args, src, img_base[0])
    def set_img_base(url):
        img_base[0] = url
        return ''


    dct = {k: wrap(v) for k,v in cfg.items()}
    dct['img_url'] = img_url
    dct['set_img_base'] = set_img_base

    with open(args.in_file, 'r') as f:
        tmpl = f.read()
    result = template(tmpl, **dct) + '\n'
    with open(args.out_file, 'w') as f:
        f.write(result)

    with open(args.out_file + '.d', 'w') as f:
        f.write('%s: \\\n' % os.path.join(args.out_file))
        for s in sorted(SRC_IMAGES):
            f.write('    %s \\\n' % s)

    with open(args.out_file + '-imgs.txt', 'w') as f:
        for s in sorted(USED_IMAGES):
            f.write('%s\n' % s)

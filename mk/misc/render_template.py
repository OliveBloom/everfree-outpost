import os
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

if __name__ == '__main__':
    sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
    from configure.template import template

    config_path, = sys.argv[1:]

    with open(config_path) as f:
        cfg = yaml.load(f)

    dct = {k: wrap(v) for k,v in cfg.items()}
    sys.stdout.write(template(sys.stdin.read(), **dct) + '\n')

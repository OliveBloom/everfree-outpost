import argparse
import json
import os
import re
import sys


def build_parser():
    args = argparse.ArgumentParser()

    args.add_argument('--file', action='append', default=[],
            help='include a file in the manifest')
    args.add_argument('--walk-js-file', action='append', default=[],
            help='include a Javascript file and all its dependencies')
    args.add_argument('--output',
            help='output file path')

    return args


REQUIRE_RE = re.compile(r'''require\(['"]([a-zA-Z0-9_/]+)['"]\)''')

def collect_deps(path):
    deps = set()
    with open(path, 'r') as f:
        for line in f:
            for match in REQUIRE_RE.finditer(line):
                deps.add(match.group(1))
    deps = sorted(deps)
    return deps

def walk_js_deps(root_path):
    root_dir = os.path.dirname(root_path)

    seen = set()
    order = []
    def walk(name):
        nonlocal root_dir, seen, order

        if name in seen:
            return
        seen.add(name)

        path = os.path.join(root_dir, '%s.js' % name)
        deps = collect_deps(path)

        for dep in deps:
            walk(dep)
        order.append(name)

    root_file = os.path.basename(root_path)
    root_name, _, _ = root_file.partition('.')
    walk(root_name)

    return order

def main(args):
    ns = build_parser().parse_args(args)

    all_files = []

    for path in ns.file:
        all_files.append((path, os.path.basename(path)))

    for path in ns.walk_js_file:
        path, _, prefix = path.partition('::')
        dir_ = os.path.dirname(path)
        for f in walk_js_deps(path):
            f += '.js'
            all_files.append((os.path.join(dir_, f), os.path.join(prefix, f)))

    total_size = sum(os.stat(f).st_size for f,_ in all_files)

    with open(ns.output, 'w') as f:
        json.dump({
            'files': [rel for (_, rel) in all_files],
            'total_size': total_size,
            'redirects': {
                'configedit': 'configedit.html',
                },
            }, f)

    with open(ns.output + '.d', 'w') as f:
        f.write('%s:\\\n' % ns.output)
        for dep, _ in all_files:
            f.write('    %s\\\n' % dep)

if __name__ == '__main__':
    main(sys.argv[1:])

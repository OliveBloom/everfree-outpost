import argparse
import json
import os
import struct
import sys


import binary_defs.context
import binary_defs.client
import binary_defs.server


# Increment when file/section header format changes
VER_MAJOR = 2

# Minor version is separate between client and server


def header(ctx, ver_minor):
    header = bytearray()
    header.extend(struct.pack('<HHIII',
        ver_minor, VER_MAJOR, len(ctx.sections), 0, 0))

    # Start section offsets after the file and section headers
    offset = 16 * (1 + len(ctx.sections))

    for name, sect in sorted(ctx.sections.items()):
        size = len(sect)

        # Section header format:
        #   u8[8] name
        #   u32 offset
        #   u32 byte_len
        header.extend(struct.pack('<8sII',
            name, offset, size))

        # Align to 8 bytes
        offset = (offset + size + 7) & ~7

    return header

def chunks(ctx, ver_minor):
    yield header(ctx, ver_minor)

    offset = 16 * (1 + len(ctx.sections))
    for name, sect in sorted(ctx.sections.items()):
        yield sect
        offset += len(sect)

        # Align to 8 bytes
        adj = -offset & 7
        if adj > 0:
            yield b'\0' * adj
            offset += adj

def write_server(ctx, defs, f):
    binary_defs.server.convert(ctx, defs)

    for c in chunks(ctx, binary_defs.server.VER_MINOR):
        f.write(c)

def write_client(ctx, defs, f):
    binary_defs.client.convert(ctx, defs)

    for c in chunks(ctx, binary_defs.client.VER_MINOR):
        f.write(c)


def build_parser():
    args = argparse.ArgumentParser()

    args.add_argument('mode', choices=('client', 'server'),
            help='output mode')
    args.add_argument('input', metavar='DATA_DIR',
            help='input data directory (containing JSON files)')
    args.add_argument('output', metavar='FILE_OUT.bin',
            help='output file (binary-formatted)')

    args.add_argument('--gen-phf', metavar='path', default='gen_phf',
            help='path to gen_phf binary')

    return args

def main():
    args = build_parser().parse_args()

    # Collect input files
    if args.mode == 'server':
        files = binary_defs.server.FILES
    elif args.mode == 'client':
        files = binary_defs.client.FILES
    else:
        assert False, 'unsupported mode %r' % args.mode

    defs = {}
    deps = set()
    for k in files:
        path = os.path.join(args.input, '%s_%s.json' % (k, args.mode))
        deps.add(path)
        with open(path) as f:
            defs[k] = json.load(f)

    # Generate and write output
    ctx = binary_defs.context.Context(gen_phf=args.gen_phf)
    with open(args.output, 'wb') as f:
        if args.mode == 'client':
            write_client(ctx, defs, f)
        elif args.mode == 'server':
            write_server(ctx, defs, f)
        else:
            assert False, 'bad mode: %r' % args.mode

    # Collect additional deps based on imported modules
    for k,v in sys.modules.items():
        if k.partition('.')[0] == 'binary_defs':
            f = getattr(v, '__file__', None)
            if f is not None:
                deps.add(f)

    # Write deps to file
    with open(args.output + '.d', 'w') as f:
        f.write('%s: \\\n' % args.output)
        for x in sorted(deps):
            f.write('  %s \\\n' % x)
        f.write('\n')

if __name__ == '__main__':
    main()

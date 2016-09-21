import argparse
import json
import os
import struct


import binary_defs.context
import binary_defs.client
import binary_defs.server


# Increment when file/section header format changes
VER_MAJOR = 2

# Minor version is separate between client and server


def header(ctx, ver_minor):
    header = bytearray()
    header.extend(struct.pack('<HHIII',
        VER_MAJOR, ver_minor, len(ctx.sections), 0, 0))

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

def write_server(defs, f):
    ctx = binary_defs.context.Context()
    binary_defs.server.convert(ctx, defs)

    for c in chunks(ctx, binary_defs.server.VER_MINOR):
        f.write(c)

def write_client(defs, f):
    ctx = binary_defs.context.Context()
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

    return args

def main():
    args = build_parser().parse_args()

    if args.mode == 'server':
        files = binary_defs.server.FILES
    elif args.mode == 'client':
        files = binary_defs.client.FILES
    else:
        assert False, 'unsupported mode %r' % args.mode

    defs = {}
    for k in files:
        with open(os.path.join(args.input, '%s_%s.json' % (k, args.mode))) as f:
            defs[k] = json.load(f)

    with open(args.output, 'wb') as f:
        if args.mode == 'client':
            write_client(defs, f)
        elif args.mode == 'server':
            write_server(defs, f)
        else:
            assert False, 'bad mode: %r' % args.mode

if __name__ == '__main__':
    main()

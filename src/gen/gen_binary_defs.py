import argparse
import json
import os
from pprint import pprint
import struct

def build_parser():
    args = argparse.ArgumentParser()

    args.add_argument('--mode', required=True,
            choices=('blocks', 'item_defs', 'item_strs',
                'templates', 'template_parts', 'template_verts', 'template_shapes'),
            help='convert block defs')

    args.add_argument('input', metavar='FILE_IN.json',
            help='input file (JSON-formatted)')
    args.add_argument('output', metavar='FILE_OUT.bin',
            help='output file (binary-formatted)')

    return args

def flatten(xs):
    for x in xs:
        try:
            # Try and see if x is iterable
            for x2 in flatten(x):
                yield x2
        except:
            # x isn't iterable
            yield x

class Field:
    def __init__(self, name, ty, offset, default=None):
        self.name = name
        self.ty = ty
        self.offset = offset
        self.default = default

class Converter:
    def __init__(self, total_size, fields):
        # Sort fields by offset
        self.fields = sorted(fields, key=lambda f: f.offset)

        code = '<'
        size = 0
        for f in self.fields:
            while size < f.offset:
                code += 'x'
                size += 1

            code += f.ty
            size += struct.calcsize(f.ty)

        while size < total_size:
            code += 'x'
            size += 1

        self.code = code

    def convert(self, obj):
        def extract(f):
            val = obj.get(f.name, f.default)
            if val is None:
                pprint(obj)
                raise KeyError('object is missing required field %r' % f.name)
            return val

        values = tuple(flatten(extract(f) for f in self.fields))
        return struct.pack(self.code, *values)


def convert_blocks(j):
    c = Converter(16, (
        Field('front',      'H',  0,  0),
        Field('back',       'H',  2,  0),
        Field('top',        'H',  4,  0),
        Field('bottom',     'H',  6,  0),

        Field('light_r',    'B',  8,  0),
        Field('light_g',    'B',  9,  0),
        Field('light_b',    'B', 10,  0),
        Field('shape',      'B', 11,  0),
        Field('light_radius', 'H', 12,  0),
        ))

    b = bytearray()
    for obj in j:
        b.extend(c.convert(obj))
    return b

def convert_items(j):
    # Combine strings first
    idx_map = {}
    strs = bytearray()
    defs = bytearray()

    def intern(s):
        if s not in idx_map:
            idx_map[s] = len(strs)
            strs.extend(s.encode('utf-8'))
        return idx_map[s]

    c = Converter(16, (
        Field('name_off',       'I',  0,  0),
        Field('name_len',       'I',  4,  0),
        Field('ui_name_off',    'I',  8,  0),
        Field('ui_name_len',    'I', 12,  0),
        ))

    for obj in j:
        defs.extend(c.convert({
            'name_off': intern(obj['name']),
            'name_len': len(obj['name']),
            'ui_name_off': intern(obj['ui_name']),
            'ui_name_len': len(obj['ui_name']),
            }))

    return defs, strs


def convert_templates(j):
    c = Converter(20, (
        Field('size',           'BBB',  0),
        Field('shape_idx',      'H',    4),
        Field('part_idx',       'H',    6),
        Field('part_count',     'B',    8),
        Field('vert_count',     'B',    9),
        Field('layer',          'B',    10),
        Field('flags',          'B',    11, 0),

        Field('light_pos',      'BBB', 12, (0, 0, 0)),
        Field('light_color',    'BBB', 15, (0, 0, 0)),
        Field('light_radius',   'H',   18, 0),
        ))

    b = bytearray()
    for obj in j:
        b.extend(c.convert(obj))
    return b

def convert_template_parts(j):
    c = Converter(14, (
        Field('vert_idx',       'H',    0),
        Field('vert_count',     'H',    2),
        Field('offset',         'hh',   4),
        Field('sheet',          'B',    8),
        Field('flags',          'B',    9, 0),

        Field('anim_length',    'b',   10, 0),
        Field('anim_rate',      'B',   11, 0),
        Field('anim_step',      'H',   12, 0),
        ))

    b = bytearray()
    for obj in j:
        if 'anim_size' in obj:
            obj['anim_step'] = obj['anim_size'][0]
        b.extend(c.convert(obj))
    return b

def convert_template_verts(j):
    b = bytearray()
    for x in j:
        b.extend(struct.pack('H', x))
    return b

def convert_template_shapes(j):
    b = bytearray()
    for x in j:
        b.extend(struct.pack('B', x))
    return b


def main():
    parser = build_parser()
    args = parser.parse_args()

    with open(args.input) as f:
        j = json.load(f)

    if args.mode == 'blocks':
        b = convert_blocks(j)
    elif args.mode == 'item_defs':
        b, _ = convert_items(j)
    elif args.mode == 'item_strs':
        _, b = convert_items(j)
    elif args.mode == 'templates':
        b = convert_templates(j)
    elif args.mode == 'template_parts':
        b = convert_template_parts(j)
    elif args.mode == 'template_verts':
        b = convert_template_verts(j)
    elif args.mode == 'template_shapes':
        b = convert_template_shapes(j)
    else:
        parser.error('must provide flag to indicate input type')

    with open(args.output, 'wb') as f:
        f.write(b)

if __name__ == '__main__':
    main()

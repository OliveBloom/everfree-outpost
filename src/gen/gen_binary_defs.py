import argparse
from collections import namedtuple
import json
import os
from pprint import pprint
import struct

def build_parser():
    args = argparse.ArgumentParser()

    args.add_argument('input', metavar='DATA_DIR',
            help='input data directory (containing JSON files)')
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
        self.size = size

    def convert(self, obj):
        def extract(f):
            val = obj.get(f.name, f.default)
            if val is None:
                pprint(obj)
                raise KeyError('object is missing required field %r' % f.name)
            return val

        values = tuple(flatten(extract(f) for f in self.fields))
        return struct.pack(self.code, *values)


Section = namedtuple('Section', ('size', 'count', 'data'))

VER_MAJOR, VER_MINOR = (0, 1)

class BinaryDefs:
    def __init__(self, args):
        self.sections = {}
        self.args = args
        self.strings = bytearray()
        self.string_map = {}
        self.header = None

        self.deps = set()

    def load(self, path):
        full_path = os.path.join(self.args.input, path)
        self.deps.add(full_path)
        with open(full_path) as f:
            return json.load(f)

    def add_section(self, name, data, size, count):
        assert name not in self.sections
        assert type(name) is bytes
        assert len(name) == 8
        assert len(data) == size * count
        self.sections[name] = Section(size, count, data)

    def convert_file(self, name, path, conv, adjust=lambda x: None):
        j = self.load(path)
        self.convert_array(name, j, conv, adjust=adjust)

    def convert_array(self, name, arr, conv, adjust=lambda x: None):
        b = bytearray()
        for obj in arr:
            adjust(obj)
            b.extend(conv.convert(obj))
        self.add_section(name, b, conv.size, len(arr))

    def pack_file(self, name, path, fmt):
        j = self.load(path)
        self.pack_array(name, j, fmt)

    def pack_array(self, name, arr, fmt):
        b = bytearray()
        for x in arr:
            if isinstance(x, (tuple, list)):
                b.extend(struct.pack(fmt, *x))
            else:
                b.extend(struct.pack(fmt, x))
        self.add_section(name, b, struct.calcsize(fmt), len(arr))

    def intern(self, s):
        if s not in self.string_map:
            self.string_map[s] = len(self.strings)
            self.strings.extend(s.encode('utf-8'))
        return self.string_map[s]

    def intern_strings(self, obj):
        for k,v in list(obj.items()):
            if isinstance(v, str):
                obj[k + '_off'] = self.intern(v)
                obj[k + '_len'] = len(v)


    def finish(self):
        self.add_section(b'Strings\0', self.strings, 1, len(self.strings))

        base_offset = 16 * (1 + len(self.sections))

        self.header = bytearray()
        # Header format:
        #   u16 ver_minor
        #   u16 ver_major
        #   u32 num_sections
        #   u32 _reserved0
        #   u32 _reserved1
        self.header.extend(struct.pack('<HHIII',
            VER_MINOR, VER_MAJOR, len(self.sections), 0, 0))

        offset = 16 * (1 + len(self.sections))
        for name, sect in sorted(self.sections.items()):
            # Section header format:
            #   u8[8] name
            #   u32 offset
            #   u16 item_size
            #   u16 item_count
            self.header.extend(struct.pack('<8sIHH',
                name, offset, sect.size, sect.count))

            # Align to 8 bytes
            offset = (offset + len(sect.data) + 7) & ~7

    def chunks(self):
        yield self.header

        offset = 16 * (1 + len(self.sections))
        for name, sect in sorted(self.sections.items()):
            yield sect.data
            offset += len(sect.data)

            # Align to 8 bytes
            adj = -offset & 7
            if adj > 0:
                yield b'\0' * adj
                offset += adj


    def convert_blocks(self):
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

        self.convert_file(b'Blocks\0\0', 'blocks_client.json', c)

    def convert_items(self):
        c = Converter(16, (
            Field('name_off',       'I',  0,  0),
            Field('name_len',       'I',  4,  0),
            Field('ui_name_off',    'I',  8,  0),
            Field('ui_name_len',    'I', 12,  0),
            ))

        self.convert_file(b'Items\0\0\0', 'items_client.json', c,
                adjust=self.intern_strings)

    def convert_structures(self):
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
        self.convert_file(b'StrcDefs', 'structures_client.json', c)

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
        def adjust(obj):
            if 'anim_size' in obj:
                obj['anim_step'] = obj['anim_size'][0]
        self.convert_file(b'StrcPart', 'structure_parts_client.json', c,
                adjust=adjust)

        raw_arr = self.load('structure_verts_client.json')
        assert len(raw_arr) % 3 == 0
        arr = [raw_arr[i : i + 3] for i in range(0, len(raw_arr), 3)]
        self.pack_array(b'StrcVert', arr, '<3H')

        self.pack_file(b'StrcShap', 'structure_shapes_client.json', 'B')

    def convert_sprites(self):
        c = Converter(4, (
            Field('local_id',       'H',    0),
            Field('framerate',      'B',    2),
            Field('length',         'B',    3),
            ))
        self.convert_file(b'SprtAnim', 'animations_client.json', c)

        c = Converter(4, (
            Field('start',          'H',    0),
            Field('count',          'H',    2),
            ))
        self.convert_file(b'SprtLayr', 'sprite_layers_client.json', c)

        c = Converter(14, (
            Field('src_offset',     'HH',   0),
            Field('dest_offset',    'HH',   4),
            Field('size',           'HH',   8),
            Field('sheet',          'B',   12),
            Field('mirror',         'B',   13),
            ))
        self.convert_file(b'SprtGrfx', 'sprite_graphics_client.json', c)

    def convert_extras(self):
        j = self.load('extras_client.json')

        # XPonLayr - pony_layer_table
        self.pack_array(b'XPonLayr', j['pony_layer_table'], 'B')

        # XPhysAnm - physics_anim_table
        arr = j['physics_anim_table']
        for i in range(len(arr)):
            if arr[i] is None:
                arr[i] = [0] * 8
        self.pack_array(b'XPhysAnm', arr, '<8H')

        # XAnimDir - anim_dir_table
        max_idx = max(int(k) for k in j['anim_dir_table'].keys())
        lst = [255] * (max_idx + 1)
        for k,v in j['anim_dir_table'].items():
            lst[int(k)] = v
        self.pack_array(b'XAnimDir', lst, 'B')

        # XDefAnim - default_anim
        self.pack_array(b'XDefAnim', [j['default_anim']], '<H')

        # XEdtAnim - editor_anim
        self.pack_array(b'XEdtAnim', [j['editor_anim']], '<H')

    def convert_day_night(self):
        j = self.load('day_night.json')

        cut_0 = 0
        colors = [(255, 255, 255)]
        cut_1 = len(colors)
        colors.extend(reversed(j['sunset']))
        cut_2 = len(colors)
        colors.extend(j['sunrise'])
        cut_3 = len(colors)

        phases = [
                (j['day_end'],      cut_0,      cut_1),     # day
                (j['night_start'],  cut_1,      cut_2 - 1), # sunset
                (j['night_end'],    cut_2 - 1,  cut_2),     # night
                (24000,             cut_2,      cut_3 - 1), # sunrise
                ]

        self.pack_array(b'DyNtPhas', phases, '<HBB')
        self.pack_array(b'DyNtColr', colors, '3B')

def main():
    parser = build_parser()
    args = parser.parse_args()

    bd = BinaryDefs(args)

    bd.convert_blocks()
    bd.convert_items()
    bd.convert_structures()
    bd.convert_sprites()
    bd.convert_extras()
    bd.convert_day_night()

    bd.finish()

    with open(args.output, 'wb') as f:
        for chunk in bd.chunks():
            f.write(chunk)

    with open(args.output + '.d', 'w') as f:
        f.write('%s: \\\n' % args.output)
        for x in sorted(bd.deps):
            f.write('  %s \\\n' % x)
        f.write('\n')

if __name__ == '__main__':
    main()

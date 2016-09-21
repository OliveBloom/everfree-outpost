from collections import defaultdict
import os
import struct
import subprocess


class Conversion:
    def size(self):
        return struct.calcsize(self.format())

    def convert(self, ctx, x):
        return struct.pack('<' + self.format(), *self.values(ctx, x))

class Scalar(Conversion):
    def __init__(self, ty):
        self.ty = ty

    def align(self):
        return struct.calcsize(self.ty)

    def format(self):
        return self.ty

    def values(self, ctx, x):
        return (x,)

class Vector(Conversion):
    def __init__(self, ty, count):
        self.ty = ty
        self.count = count

    def align(self):
        return struct.calcsize(self.ty)

    def format(self):
        return self.ty * self.count

    def values(self, ctx, x):
        return tuple(x)

class String(Conversion):
    def __init__(self, section=b'Strings\0', idx_ty='I'):
        self.section = section
        self.idx_ty = idx_ty

    def align(self):
        return struct.calcsize(self.idx_ty)

    def format(self):
        return self.idx_ty * 2

    def values(self, ctx, x):
        b = x.encode('utf-8')
        offset = ctx.intern(self.section, b)
        return (offset, len(x))

class Sequence(Conversion):
    def __init__(self, section, conv, idx_ty='I', offset_only=False):
        self.section = section
        self.conv = conv
        self.idx_ty = idx_ty
        self.offset_only = offset_only

    def align(self):
        return struct.calcsize(self.idx_ty)

    def format(self):
        if not self.offset_only:
            return self.idx_ty * 2
        else:
            return self.idx_ty

    def values(self, ctx, x):
        b = b''.join(self.conv.convert(ctx, y) for y in x)
        offset = ctx.intern(self.section, b)
        if not self.offset_only:
            return (offset, len(x))
        else:
            return (offset,)


class Field:
    def __init__(self, key, conv, default=None, offset=None):
        self.key = key
        self.conv = conv
        self.default = default
        self.offset = offset

class Struct(Conversion):
    def __init__(self, fields, size=None):
        self.fields = fields

        offset = 0
        max_align = 0
        code = ''

        for f in self.fields:
            if f.offset is None:
                align = f.conv.align()
                f.offset = (offset + align - 1) & ~(align - 1)

            while offset < f.offset:
                code += 'x'
                offset += 1

            code += f.conv.format()
            offset += f.conv.size()
            max_align = max(max_align, f.conv.align())

        if size is None:
            align = max_align
            size = (offset + align - 1) & ~(align - 1)

        while offset < size:
            code += 'x'
            offset += 1

        assert struct.calcsize(code) == offset

        self._format = code
        self._align = max_align

    def align(self):
        return self._align

    def format(self):
        return self._format

    def values(self, ctx, x):
        vals = []
        for f in self.fields:
            if f.key not in x and f.default is None:
                raise KeyError(f.key)
            y = x.get(f.key, f.default)
            vals.extend(f.conv.values(ctx, y))
        return tuple(vals)


class Context:
    def __init__(self):
        self.sections = defaultdict(bytearray)
        self.intern_maps = defaultdict(dict)

    def intern(self, section, b):
        offset = self.intern_maps[section].get(b)
        if offset is None:
            offset = len(self.sections[section])
            self.sections[section].extend(b)
            self.intern_maps[section][b] = offset
        return offset

    def convert(self, section, conv, objs):
        assert section not in self.sections
        self.sections[section] = b''.join(conv.convert(self, obj) for obj in objs)

    def build_index(self, name, strs, idx_ty='H'):
        p = subprocess.Popen((os.environ['OUTPOST_BUILD_PHF'],),
                stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        result, _ = p.communicate(''.join(s + '\n' for s in strs).encode('utf-8'))
        assert p.wait() == 0, 'hash builder failed'
        lines = result.splitlines()

        b, m, r = [int(x) for x in lines[0].split()[1:]]
        hashes = [int(x) for x in lines[1].split()[1:]]
        params = [int(x) for x in lines[2].split()[1:]]

        dummy, = struct.unpack('<' + idx_ty, b'\xff' * struct.calcsize(idx_ty))
        table = [dummy] * b
        for i, h in enumerate(hashes):
            table[h] = i
        table_bytes = b''.join(struct.pack('<' + idx_ty, x) for x in table)
        self.sections[b'IxTb' + name] = table_bytes

        # `b` implicit in the length of the object list, and `r` is the length
        # of `params`.  Only `m` needs to be stored explicitly.
        param_header = struct.pack('<I', m)
        param_body = b''.join(struct.pack('<' + idx_ty, x) for x in params)
        self.sections[b'IxPr' + name] = param_header + param_body




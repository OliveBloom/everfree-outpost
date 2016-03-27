from collections import namedtuple
import json
import os
import subprocess
import sys
import textwrap
import time

from PIL import Image


Band = namedtuple('Band', ('x', 'y', 'w', 'h'))

def consume(band, w, h):
    '''Remove a `w`-by-`h` chunk from the top-left of the band.  Return a list
    of bands covering the leftover space (if any) to the bottom and right.'''
    assert w <= band.w and h <= band.h

    result = []

    top = Band(band.x, band.y, band.w, h)
    if h < band.h:
        result.append(Band(band.x, band.y + h, band.w, band.h - h))

    # For reference:
    #left = Band(top.x, top.y, w, top.h)
    if w < top.w:
        result.append(Band(top.x + w, top.y, top.w - w, top.h))

    return result

class Bands:
    def __init__(self, w, h):
        self.bands = [Band(0, 0, w, h)]

    def place(self, w, h):
        best = None
        best_idx = None
        for i, band in enumerate(self.bands):
            if band.w < w or band.h < h:
                continue
            if best is None:
                best = band
                best_idx = i
                continue
            if band.h < best.h or (band.h == best.h and band.w < best.w):
                best = band
                best_idx = i

        if best is None:
            raise RuntimeError('no space for an object of size %d x %d' % (w, h))

        self.bands[best_idx] = self.bands[-1]
        self.bands.pop()
        self.bands.extend(consume(best, w, h))
        return (best.x, best.y)


class Part:
    def __init__(self, name, path, img):
        self.name = name
        self.path = path
        self.img = img
        self.x, self.y = None, None
        self.w, self.h = img.size


def load_parts(src_dir):
    parts = []
    for f in os.listdir(src_dir):
        if f.endswith('.png'):
            name, _ = os.path.splitext(f)
            path = os.path.join(src_dir, f)
            img = Image.open(path)
            parts.append(Part(name, path, img))

    return parts

def place_parts(parts, size):
    '''Arrange all `parts` into an atlas of size `size`.  Set the `x` and `y`
    fields of each `Part` to its position in the atlas.'''
    # Sort by decreasing height, then by decreasing width, and finally by name
    # (for determinism)
    parts = sorted(parts, key=lambda p: (-p.h, -p.w, p.name))
    bands = Bands(*size)

    for p in parts:
        p.x, p.y = bands.place(p.w, p.h)

def build_atlas(parts, size):
    img = Image.new('RGBA', size)

    for p in parts:
        img.paste(p.img, (p.x, p.y))

    return img

def build_json(parts):
    result = {}

    for p in parts:
        result[p.name] = dict(
                x=p.x,
                y=p.y,
                w=p.w,
                h=p.h,
                )

    return result

def build_rust(parts):
    result = ''

    now = time.strftime('%Y-%m-%d %H:%M:%S')
    result += '// Generated %s by %s\n' % (now, sys.argv[0])

    result += textwrap.dedent('''
        #![crate_name = "client_ui_atlas"]
        #![no_std]
        //! This auto-generated library defines the position and size of every
        //! element in the UI atlas, so they can be referred to by name.

        extern crate physics;
        use physics::v3::V2;

        #[derive(Clone, Copy)]
        pub struct AtlasEntry {
            pub pos: (u16, u16),
            pub size: (u8, u8),
        }

        impl AtlasEntry {
            pub fn pos(&self) -> V2 {
                V2::new(self.pos.0 as i32,
                        self.pos.1 as i32)
            }

            pub fn size(&self) -> V2 {
                V2::new(self.size.0 as i32,
                        self.size.1 as i32)
            }
        }

    ''')

    tmpl = 'pub const %s: AtlasEntry = AtlasEntry { pos: (%d, %d), size: (%d, %d) };\n'
    for p in sorted(parts, key=lambda p: p.name):
        result += tmpl % (p.name.upper().replace('-', '_'), p.x, p.y, p.w, p.h)

    return result

# Works for now, will need to increase later
ATLAS_SIZE = (256, 256)

def main(src_dir, dest_dir):
    parts = load_parts(src_dir)
    place_parts(parts, ATLAS_SIZE)

    def path(ext):
        return os.path.join(dest_dir, 'ui_atlas.%s' % ext)

    atlas = build_atlas(parts, ATLAS_SIZE)
    atlas.save(path('png'))

    j = build_json(parts)
    with open(path('json'), 'w') as f:
        json.dump(j, f)

    rs = build_rust(parts)
    with open(path('rs'), 'w') as f:
        f.write(rs)

    dep_str = ' \\\n    '.join(sorted(p.path for p in parts))
    with open(path('d'), 'w') as f:
        f.write('%s: \\\n    %s\n' % (path('png'), dep_str))

if __name__ == '__main__':
    src_dir, dest_dir = sys.argv[1:]
    main(src_dir, dest_dir)

from collections import namedtuple

from outpost_data.core import image2
from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.image2 import loader, Anim
from outpost_data.core import structure
from outpost_data.outpost.lib import meshes


def describe(x):
    """Return a string describing the part of `x` whose values are all
    `True`.  All `True` values should be connected horizontally or vertically.

    Possible outcomes:

        ##  half/n
        __

        __  inner/nw
        _#

        ##  outer/nw
        #_

        ##  full
        ##

        __  empty
        __
    
    """
    nw, ne, se, sw = x
    count = sum(int(bool(b)) for b in x)
    if count == 0:
        return 'empty'
    elif count == 1:
        if nw: side = 'se'
        if ne: side = 'sw'
        if se: side = 'nw'
        if sw: side = 'ne'
        return 'inner/%s' % side
    elif count == 2:
        if nw and ne: side = 'n'
        if ne and se: side = 'e'
        if se and sw: side = 's'
        if sw and nw: side = 'w'
        if nw and se: return 'cross/nw'
        if ne and sw: return 'cross/ne'
        return 'half/%s' % side
    elif count == 3:
        if not nw: side = 'se'
        if not ne: side = 'sw'
        if not se: side = 'nw'
        if not sw: side = 'ne'
        return 'outer/%s' % side
    elif count == 4:
        return 'full'

def dissect(x, count):
    """Describe the area covered by each value in `x`."""
    areas = []
    for v in range(count):
        desc = describe(tuple(int(w == v) for w in x))
        areas.append(desc)
    return areas

def dissect_layered(x, count):
    areas = []
    for v in range(count):
        desc = describe(tuple(int(w >= v) for w in x))
        areas.append(desc)
    return areas

def iter_codes(n):
    for a in range(n):
        for b in range(n):
            for c in range(n):
                for d in range(n):
                    yield (a, b, c, d)


# Image manipulation

OUTER_MIN = 0
OUTER_MAX = 3 * TILE_SIZE
INNER_MIN = OUTER_MIN + TILE_SIZE // 2
INNER_MAX = OUTER_MAX - TILE_SIZE // 2

def clear_center(img):
    """Erase the center part of a 3x3-tile image."""
    def f(raw):
        color = (0, 0, 0, 0)
        bounds = (INNER_MIN, INNER_MIN, INNER_MAX, INNER_MAX)
        raw.paste(color, bounds)

    return img.modify(f)

def clear_border(img):
    """Erase the border of a 3x3-tile image."""
    def f(raw):
        color = (0, 0, 0, 0)
        # left
        raw.paste(color, (OUTER_MIN, OUTER_MIN, INNER_MIN, OUTER_MAX))
        # right
        raw.paste(color, (INNER_MAX, OUTER_MIN, OUTER_MAX, OUTER_MAX))
        # top
        raw.paste(color, (INNER_MIN, OUTER_MIN, INNER_MAX, INNER_MIN))
        # bottom
        raw.paste(color, (INNER_MIN, INNER_MAX, INNER_MAX, OUTER_MAX))

    return img.modify(f)

def blank_left(raw):
    raw.paste((0, 0, 0, 0), (0, 0, TILE_SIZE // 2, TILE_SIZE))
def blank_right(raw):
    raw.paste((0, 0, 0, 0), (TILE_SIZE // 2, 0, TILE_SIZE, TILE_SIZE))

def add_cross(parts):
    parts['cross/nw'] = image2.stack((parts['inner/nw'], parts['inner/se']))
    parts['cross/ne'] = image2.stack((parts['inner/ne'], parts['inner/sw']))


BORDER_PARTS = (
        ('outer/nw', 'half/n', 'outer/ne'),
        ('half/w',   None,     'half/e'),
        ('outer/sw', 'half/s', 'outer/se'),
        )

CENTER_PARTS = (
        ('inner/nw', None,     'inner/ne'),
        (None,       None,     None),
        ('inner/sw', None,     'inner/se'),
        )

BLANK_TILE = image2.Image(size=(1, 1), unit=TILE_SIZE)

def chop_cave_top(img):
    """Build a dict of cave-top sections."""
    img = img.extract((0, 0), (3, 3))

    parts = {}
    parts.update(clear_center(img).chop_grid(BORDER_PARTS))
    parts.update(clear_border(img).chop_grid(CENTER_PARTS))
    parts['full'] = BLANK_TILE
    parts['empty'] = BLANK_TILE
    add_cross(parts)
    return parts

FRONT_PARTS = (
        ('outer/w/z1',  'center/z1',    'outer/e/z1'),
        ('outer/w/z0',  'center/z0',    'outer/e/z0'),
        )

def chop_cave_front(img):
    """Build a dict of cave front-wall sections."""
    img = img.extract((0, 3), (3, 2))

    parts = img.chop_grid(FRONT_PARTS)

    for z in (0, 1):
        parts['inner/w/z%d' % z] = parts['center/z%d' % z].modify(blank_left)
        parts['inner/e/z%d' % z] = parts['center/z%d' % z].modify(blank_right)

    return parts

def chop_cave_entrance(img):
    parts = {'left': {}, 'center': {}, 'right': {}}
    for z in (0, 1):
        # Left
        e = img.extract((0, 6 - z))
        w = img.extract((0, 4 - z))
        e_half = e.modify(blank_left)
        w_half = w.modify(blank_right)
        parts['left'].update({
            'outer/w/z%d' % z: image2.stack((e_half, w_half)),
            'inner/w/z%d' % z: e_half,
            'center/z%d' % z: e,
            })

        # Right
        e = img.extract((2, 6 - z))
        w = img.extract((2, 4 - z))
        e_half = e.modify(blank_right)
        w_half = w.modify(blank_left)
        parts['right'].update({
            'outer/e/z%d' % z: image2.stack((e_half, w_half)),
            'inner/e/z%d' % z: e_half,
            'center/z%d' % z: e,
            })

        # Center
        e = img.extract((1, 6 - z))
        parts['center']['center/z%d' % z] = e

    return parts

# The color used for filled areas inside caves.
BLACK = (50, 33, 37)

def chop_black():
    """Build a dict of solid black sections."""
    img = image2.Image(size=(3, 3), unit=TILE_SIZE)
    img = img.modify(lambda i: i.paste(BLACK + (255,)))

    parts = {}
    parts.update(clear_center(img).chop_grid(BORDER_PARTS))
    parts.update(clear_border(img).chop_grid(CENTER_PARTS))
    parts['full'] = img.extract((0, 0))
    parts['empty'] = BLANK_TILE
    add_cross(parts)
    return parts

RAMP_TOP_PARTS = (
       ('inner/nw',     'half/s',       'inner/ne'),
       ('outer/se',     None,           'outer/sw'),
        )

def chop_ramp_top(cave, ramp):
    def blank_outside(raw):
        color = (0, 0, 0, 0)
        raw.paste(color, (0, 0, 16, 48))
        raw.paste(color, (80, 0, 96, 48))
        raw.paste(color, (16, 0, 80, 16))
    ramp = ramp.modify(blank_outside)

    cave_parts = clear_center(cave).chop_grid(BORDER_PARTS)

    parts = ramp.chop_grid(RAMP_TOP_PARTS)
    parts['half/e'] = image2.stack((
            parts['outer/se'].modify(blank_left),
            cave_parts['half/e']))
    parts['half/w'] = image2.stack((
            parts['outer/sw'].modify(blank_right),
            cave_parts['half/w']))
    return parts

def chop_ramp_front(cave, ramp):
    cave_parts = chop_cave_front(cave)
    left_parts = cave_parts.copy()
    right_parts = cave_parts.copy()

    for z in (0, 1):
        left_img = ramp.extract((0, 3 - z)).modify(blank_left)
        for k, v in left_parts.items():
            if k.endswith('/z%d' % z):
                left_parts[k] = image2.stack((v, left_img))
        left_parts['empty/z%d' % z] = left_img

        right_img = ramp.extract((2, 3 - z)).modify(blank_right)
        for k, v in right_parts.items():
            if k.endswith('/z%d' % z):
                right_parts[k] = image2.stack((v, right_img))
        right_parts['empty/z%d' % z] = right_img

    return {
            'left': left_parts,
            'right': right_parts,
            }

# NB: If this looks backwards, it's because the name describes which parts of
# the tile are covered by the terrain.  So 'half/n' means the north half is
# covered, and that cell appears to the south of the center.
TERRAIN_PARTS = (
       (None,       'outer/nw',     'outer/ne'),
       (None,       'outer/sw',     'outer/se'),
       ('inner/nw', 'half/s',       'inner/ne'),
       ('half/e',   'full',         'half/w'),
       ('inner/sw', 'half/n',       'inner/se'),
       )

def chop_terrain(img, cross_img=None):
    parts = img.chop_grid(TERRAIN_PARTS)
    parts['empty'] = BLANK_TILE
    if cross_img is not None:
        parts.update(cross_img.chop_grid((
            ('cross/nw',),
            ('cross/ne',),
            )))
    else:
        parts['cross/nw'] = image2.stack((parts['inner/nw'], parts['inner/se']))
        parts['cross/ne'] = image2.stack((parts['inner/ne'], parts['inner/sw']))
    return parts


# Build dicts of block parts

def calc_front_desc(ds):
    # Get front tile type
    need_w = False
    need_e = False

    for j in (1, 2):
        if ds[j] == 'outer/sw':
            return 'outer/w'
        elif ds[j] == 'outer/se':
            return 'outer/e'
        elif ds[j] == 'inner/nw' or ds[j] == 'cross/nw':
            need_w = True
        elif ds[j] == 'inner/ne' or ds[j] == 'cross/ne':
            need_e = True
        elif ds[j] == 'half/s':
            return 'center'

    if need_w and need_e:
        return 'center'
    elif need_w:
        return 'inner/w'
    elif need_e:
        return 'inner/e'

    return None

#def calc_bottom_desc(i):
    #return describe(tuple(x == 1 for x in unpack4(i, 3)))

def chop_cave_front_codes(img, z):
    front_dct = chop_cave_front(img)
    result = {}

    for code in iter_codes(3):
        ds = dissect(code, 3)

        front_desc = calc_front_desc(ds)
        if front_desc is None:
            continue
        result[code] = front_dct['%s/z%d' % (front_desc, z)]

    return result

def chop_terrain_codes(img):
    dct = chop_terrain(img)
    result = {}

    for code in iter_codes(2):
        desc = describe(code)
        result[code] = dct[desc]

    return result


# Build tiles from dicts

Layer = namedtuple('Layer', ('name', 'dct', 'is_base'))

def iter_terrain_floor(layers):
    letters = [l.name for l in layers]
    for code in iter_codes(len(layers)):
        name = ''.join(layers[i].name for i in code)

        parts = []
        for i in range(len(layers)):
            l = layers[i]
            if l.is_base:
                key = describe(tuple(x >= i for x in code))
            else:
                key = describe(tuple(x == i for x in code))
            parts.append(l.dct[key])
        img = image2.stack(parts)
        yield name, img


def init():
    tiles = loader('tiles', unit=TILE_SIZE)

    def mk_layer(letter, img_name, is_base=True):
        return Layer(letter, chop_terrain(tiles(img_name)), is_base)

    layer_dct = {
            'g': mk_layer('g', 'lpc-base-tiles/grassalt.png'),
            'm': mk_layer('m', 'lpc-base-tiles/dirt.png'),
            'c': mk_layer('c', 'lpc-base-tiles/dirt2.png'),
            #'s': mk_layer('s', 'TODO'),
            'a': mk_layer('a', 'lpc-base-tiles/lavarock.png'),

            'w': mk_layer('w', 'lpc-base-tiles/water.png', is_base=False),
            'l': mk_layer('l', 'lpc-base-tiles/lava.png', is_base=False),
            'p': mk_layer('p', 'lpc-base-tiles/holemid.png', is_base=False),
            }

    order = 'mca wlp g s'

    def collect_layers(letters):
        return [layer_dct[l] for l in order if l in letters]

    # Base terrain
    floor_bb = BLOCK.prefixed('terrain').shape('floor')
    seen = set()
    for letters in ('gc', 'gw'):
        for name, img in iter_terrain_floor(collect_layers(letters)):
            # Avoid duplicate blocks
            if name in seen:
                continue
            seen.add(name)

            floor_bb.new(name).bottom(img)

    # Grass hilltop edges
    grass_top = chop_terrain(tiles('cave-top-grass.png'), tiles('cave-top-grass-cross.png'))
    for code in iter_codes(2):
        d = describe(tuple(x == 0 for x in code))
        floor_bb.new('gggg/e%s' % ''.join(str(x) for x in code)).bottom(grass_top[d])

    # Cave walls
    cave_top_dct = chop_cave_top(tiles('lpc-cave-walls2.png'))
    cave_front_dct = chop_cave_front(tiles('lpc-cave-walls2.png'))
    black_dct = chop_black()
    for code in iter_codes(3):
        ds = dissect(code, 3)
        name = ''.join(str(x) for x in code)

        front_desc = calc_front_desc(ds)
        front_z0 = cave_front_dct['%s/z0' % front_desc] if front_desc is not None else None
        front_z1 = cave_front_dct['%s/z1' % front_desc] if front_desc is not None else None

        top = image2.stack((
            black_dct[ds[0]],
            cave_top_dct[ds[1]],
            cave_top_dct[ds[2]],
            ))

        clear = name in ('1111', '2222')
        BLOCK.new('terrain/gggg/c%s' % name) \
                .shape('empty' if clear else 'solid') \
                .bottom(grass_top['full']).front(front_z0)
        BLOCK.new('cave_z1/%s' % name) \
                .shape('floor' if clear else 'solid') \
                .top(top).front(front_z1)

        empty_name = ''.join(str(int(x == 1)) for x in code)
        BLOCK.new('terrain/gggg/e%s/c%s' % (empty_name, name)) \
                .shape('empty' if clear else 'solid') \
                .front(front_z0)


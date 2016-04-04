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

# NB: If this looks backwards, it's because the name describes which parts of
# the tile are covered by the terrain.  So 'half/n' means the north half is
# covered, and that cell appears to the south of the center.
TERRAIN_PARTS = (
       (None,       'outer/nw',     'outer/ne'),
       (None,       'outer/sw',     'outer/se'),
       ('inner/nw', 'half/s',       'inner/ne'),
       ('half/e',   'full',         'half/w'),
       ('inner/sw', 'half/n',       'inner/se'),
       ('full/v1',  'full/v2',      'full/v3'),
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

FloorInfo = namedtuple('FloorInfo', ('name', 'bottom'))
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
        yield FloorInfo(name, img)

CaveInfo = namedtuple('CaveInfo', ('name', 'front', 'clear'))
def iter_cave_z0(cave_img):
    cave_front_dct = chop_cave_front(cave_img)

    for code in iter_codes(3):
        ds = dissect(code, 3)
        name = ''.join(str(x) for x in code)

        front_desc = calc_front_desc(ds)
        front = cave_front_dct['%s/z0' % front_desc] if front_desc is not None else None

        clear = name in ('1111', '2222')
        yield CaveInfo(name, front, clear)


# Cave ramps are tricky because we want them to integrate well with the
# surrounding walls.  The current solution is to mostly treat the two grid
# intersections at the back of the ramp as 0 (black), producing a 3x2 block of
# wall (possibly merged into surrounding walls), but then use custom tiles at
# the front so the ramp looks like it was cut into the wall.
#
# Freestanding:         With nearby walls:
#  2   2   2   2         1   1|  0   0
#   /---------\          -----/
#   |         |                     
#   | /-----\ |          -\ /-----\
#  2| |0   0| |2         2| |0   0|  0
#   | | ramp| |          -/ | ramp|  
#   | | here| |             | here|  
#   \-/ - - \-/             | - - \---
#  2   2   2   2         0  |2   2   2
#
# For the five wall tiles adjacent to the 0s (the ramp itself is excluded), the
# outer half is rendered normally:
#
# Freestanding:         With nearby walls:
#  2   2   2   2         1   1|  0   0
#   /---------\          -----/
#   |         |                     
#   |         |          -\        
#  2|  0   0  |2         2|  0   0   0
#   |         |          -/          
#   |         |                      
#   \---------/             /---------
#  2   2   2   2         0  |2   2   2
#
# But the inner half is rendered according to a modified view of the terrain:
#
# Freestanding:         With nearby walls:
#  2|  *   *  |2         1|  *   *   0
#  -/         \-         -/    
#                                   
#     /-----\               /-----\
#  *  |#   #|  *         *  |#   #|  *
#     |     |               |     |  
#     |     |               |     |  
#  ---/     \---            |     \---
#  2   2   2   2         0  |2   2   2
#
# ('*' tiles are overridden to 0, and '#' tiles are 2.)
#
# Merging the two halves produces the combined view shown above.  Note that
# this results in some wall shapes that are otherwise impossible, such as the
# U-shaped tiles in the freestanding example.

CAVE_RAMP_SIDE = (
        'top/back/%(side)s',
        'top/front/%(side)s/corner',
        'front/%(side)s/z1',
        'front/%(side)s/z0',
        'top/front/%(side)s/edge',
        None,
        )

CAVE_RAMP_CENTER = (
        'top/back/center/%(terrain)s',
        'ramp/%(terrain)s/0',
        'ramp/%(terrain)s/1',
        'ramp/%(terrain)s/2',
        'ramp/%(terrain)s/3',
        None,
        )

CAVE_RAMP_CAPS = (
        'ramp/grass/cap0', 'ramp/grass/cap1',
        'ramp/dirt/cap0', 'ramp/dirt/cap1',
        'ramp/dirt2/cap0', 'ramp/dirt2/cap1',
        )

def _format(s, dct):
    if s is None:
        return None
    else:
        return s % dct

CAVE_RAMP_PARTS = tuple(zip(
    (_format(x, {'side': 'left'}) for x in CAVE_RAMP_SIDE),
    (_format(x, {'terrain': 'dirt'}) for x in CAVE_RAMP_CENTER),
    (_format(x, {'side': 'right'}) for x in CAVE_RAMP_SIDE),
    (_format(x, {'terrain': 'grass'}) for x in CAVE_RAMP_CENTER),
    (_format(x, {'terrain': 'dirt2'}) for x in CAVE_RAMP_CENTER),
    CAVE_RAMP_CAPS,
    ))

def do_cave_ramps(tiles, cave_bottom_dct):
    ramp = tiles('cave-stair.png').chop_grid(CAVE_RAMP_PARTS)
    wall_top = chop_cave_top(tiles('lpc-cave-walls2.png'))
    wall_front = chop_cave_front(tiles('lpc-cave-walls2.png'))
    black = chop_black()
    cave_bottom = cave_bottom_dct['cccc']

    bb = BLOCK.child().shape('solid')

    for code in iter_codes(3):
        nw, ne, se, sw = code
        ds = dissect(code, 3)
        name = ''.join(str(x) for x in code)
        terrain = ''.join(('g' if x == 1 else 'c') for x in code)

        def get_front(z, side):
            front_desc = calc_front_desc(ds)
            ramp_front = ramp['front/%s/z%d' % (side, z)]
            if front_desc is not None:
                return image2.stack((wall_front['%s/z%d' % (front_desc, z)], ramp_front))
            else:
                return ramp_front

        def get_top(side):
            return image2.stack((
                black[ds[0]],
                wall_top[ds[1]],
                wall_top[ds[2]],
                ramp['top/%s' % side],
                ))

        if ne == 0 and se != 0:
            # Front left
            if terrain != 'cccc':
                bb.new('ramp/xy01/z0/cccc/c%s' % name).front(get_front(0, 'left')) \
                        .bottom(cave_bottom)
            bb.new('ramp/xy01/z0/%s/c%s' % (terrain, name)).front(get_front(0, 'left')) \
                    .bottom(cave_bottom_dct[terrain])
            bb.new('ramp/xy01/z1/c%s' % name).front(get_front(1, 'left')) \
                    .top(get_top('front/left/%s' % ('corner' if sw != 0 else 'edge')))

        if nw == 0 and sw != 0:
            # Front right
            if terrain != 'cccc':
                bb.new('ramp/xy21/z0/cccc/c%s' % name).front(get_front(0, 'right')) \
                        .bottom(cave_bottom)
            bb.new('ramp/xy21/z0/%s/c%s' % (terrain, name)).front(get_front(0, 'right')) \
                    .bottom(cave_bottom_dct[terrain])
            bb.new('ramp/xy21/z1/c%s' % name).front(get_front(1, 'right')) \
                    .top(get_top('front/right/%s' % ('corner' if se != 0 else 'edge')))

        def normal_front(z):
            front_desc = calc_front_desc(ds)
            if front_desc is not None:
                return wall_front['%s/z%d' % (front_desc, z)]
            else:
                return None

        if se == 0:
            # Back right
            if terrain != 'cccc':
                bb.new('ramp/xy00/z0/cccc/c%s' % name).front(normal_front(0)) \
                        .bottom(cave_bottom)
            bb.new('ramp/xy00/z0/%s/c%s' % (terrain, name)).front(normal_front(0)) \
                    .bottom(cave_bottom_dct[terrain])
            bb.new('ramp/xy00/z1/c%s' % name).front(normal_front(1)) \
                    .top(get_top('back/left'))

        if sw == 0:
            # Back left
            if terrain != 'cccc':
                bb.new('ramp/xy20/z0/cccc/c%s' % name).front(normal_front(0)) \
                        .bottom(cave_bottom)
            bb.new('ramp/xy20/z0/%s/c%s' % (terrain, name)).front(normal_front(0)) \
                    .bottom(cave_bottom_dct[terrain])
            bb.new('ramp/xy20/z1/c%s' % name).front(normal_front(1)) \
                    .top(get_top('back/right'))

        if sw == 0 and se == 0:
            # Back center
            if terrain != 'cccc':
                bb.new('ramp/xy10/z0/cccc/c%s' % name).front(normal_front(0)) \
                        .bottom(cave_bottom)
            bb.new('ramp/xy10/z0/%s/c%s' % (terrain, name)).front(normal_front(0)) \
                    .bottom(cave_bottom_dct[terrain])
            bb.new('ramp/xy10/z1/c%s' % name).front(normal_front(1)) \
                    .top(get_top('back/center/dirt2'))

    bb_ramp = BLOCK.child().shape('ramp_n')
    for terrain in ('grass', 'dirt', 'dirt2'):
        bottom = cave_bottom if terrain != 'grass' else cave_bottom_dct['gggg']

        bb_ramp.new('ramp/%s/z0' % terrain) \
                .bottom(image2.stack((
                    bottom,
                    ramp['ramp/%s/3' % terrain],
                    ))) \
                .back(ramp['ramp/%s/2' % terrain])
        bb_ramp.new('ramp/%s/z1' % terrain) \
                .bottom(ramp['ramp/%s/1' % terrain]) \
                .back(ramp['ramp/%s/0' % terrain])

        BLOCK.new('ramp/%s/cap0' % terrain) \
                .shape('floor') \
                .bottom(ramp['ramp/%s/cap0' % terrain])
        BLOCK.new('ramp/%s/cap1' % terrain) \
                .shape('empty') \
                .bottom(ramp['ramp/%s/cap1' % terrain])


# Cave entrances are relatively simple.  The three tiles need to have this
# shape:
#
#   x-2-2-x
#   | | | |
#   x-1-1-x
#
# Aside from the middle block, which is completely custom, there are also
# tweaks to the left and right blocks: their front faces always show the inner
# half of the left/right column of the doorway, layered over top of any other
# front face that may be present.

def chop_cave_entrance(img):
    img = img.extract((0, 5), (3, 2))

    parts = {}

    parts['left/z1'] = img.extract((0, 0)).modify(blank_left)
    parts['left/z0'] = img.extract((0, 1)).modify(blank_left)

    parts['center/z1'] = img.extract((1, 0))
    parts['center/z0'] = img.extract((1, 1))

    parts['right/z1'] = img.extract((2, 0)).modify(blank_right)
    parts['right/z0'] = img.extract((2, 1)).modify(blank_right)

    return parts

def do_cave_entrance(tiles, cave_bottom_dct):
    wall_top = chop_cave_top(tiles('lpc-cave-walls2.png'))
    wall_front = chop_cave_front(tiles('lpc-cave-walls2.png'))
    door_front = chop_cave_entrance(tiles('lpc-cave-walls2.png'))
    black = chop_black()
    cave_bottom = cave_bottom_dct['cccc']

    bb = BLOCK.child().shape('solid')

    for code in iter_codes(3):
        nw, ne, se, sw = code
        ds = dissect(code, 3)
        name = ''.join(str(x) for x in code)
        terrain = ''.join(('g' if x == 1 else 'c') for x in code)

        def get_front(z, side):
            front_desc = calc_front_desc(ds)
            door_tile = door_front['%s/z%d' % (side, z)]
            if front_desc is not None:
                base = wall_front['%s/z%d' % (front_desc, z)]
                if side == 'left':
                    base = base.modify(blank_right)
                elif side == 'right':
                    base = base.modify(blank_left)
                return image2.stack((base, door_tile))
            else:
                return door_tile

        top = image2.stack((
            black[ds[0]],
            wall_top[ds[1]],
            wall_top[ds[2]],
            ))

        if ne == 2 and se == 1:
            bb.new('cave_entrance/x0/z0/%s/c%s' % (terrain, name)) \
                    .front(get_front(0, 'left')) \
                    .bottom(cave_bottom_dct[terrain])
            bb.new('cave_entrance/x0/z1/c%s' % name) \
                    .front(get_front(1, 'left')) \
                    .top(top)

        if code == (2, 2, 1, 1):
            bb.new('cave_entrance/x1/z0/%s/c%s' % (terrain, name)) \
                    .shape('floor') \
                    .front(door_front['center/z0']) \
                    .bottom(cave_bottom_dct[terrain])
            bb.new('cave_entrance/x1/z1/c%s' % name) \
                    .shape('empty') \
                    .front(door_front['center/z1']) \
                    .top(top)

        if nw == 2 and sw == 1:
            bb.new('cave_entrance/x2/z0/%s/c%s' % (terrain, name)) \
                    .front(get_front(0, 'right')) \
                    .bottom(cave_bottom_dct[terrain])
            bb.new('cave_entrance/x2/z1/c%s' % name) \
                    .front(get_front(1, 'right')) \
                    .top(top)

def init():
    tiles = loader('tiles', unit=TILE_SIZE)

    def mk_layer(letter, img_name, is_base=True):
        return Layer(letter, chop_terrain(tiles(img_name)), is_base)

    layer_dct = {
            'g': mk_layer('g', 'lpc-grassalt-with-variants.png'),
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
    empty_bb = BLOCK.prefixed('terrain').shape('empty')
    seen = set()
    for letters in ('gc', 'gw'):
        for name, img in iter_terrain_floor(collect_layers(letters)):
            # Avoid duplicate blocks
            if name in seen:
                continue
            seen.add(name)

            bb = floor_bb if 'w' not in letters else empty_bb
            bb.new(name).bottom(img)
    
    # Variants
    for layer in layer_dct.values():
        name = layer.name * 4
        for i in range(4):
            key = 'full/v%d' % i if i > 0 else 'full'
            floor_bb.new('%s/v%d' % (name, i)).bottom(layer.dct[key])


    # Hilltop edges
    grass_top = chop_terrain(tiles('cave-top-grass.png'), tiles('cave-top-grass-cross.png'))
    for code in iter_codes(2):
        d = describe(tuple(x == 0 for x in code))
        blk = floor_bb.new('gggg/e%s' % ''.join(str(x) for x in code)).bottom(grass_top[d])
        if code == (1, 1, 1, 1):
            blk.shape('empty')

    # Cave top (for ramp tops inside caves)
    cave_top = chop_terrain(tiles('lpc-cave-top2.png'), tiles('lpc-cave-top2-cross.png'))
    for code in iter_codes(2):
        d = describe(tuple(x == 0 for x in code))
        blk = floor_bb.new('cccc/e%s' % ''.join(str(x) for x in code)).bottom(cave_top[d])
        if code == (1, 1, 1, 1):
            blk.shape('empty')

    # Terrain with cave walls
    for floor in iter_terrain_floor(collect_layers('gc')):
        for cave in iter_cave_z0(tiles('lpc-cave-walls2.png')):
            BLOCK.new('terrain/%s/c%s' % (floor.name, cave.name)) \
                    .bottom(floor.bottom) \
                    .front(cave.front) \
                    .shape('floor' if cave.clear else 'solid')

    # Cave walls (z1 part)
    cave_top_dct = chop_cave_top(tiles('lpc-cave-walls2.png'))
    cave_front_dct = chop_cave_front(tiles('lpc-cave-walls2.png'))
    black_dct = chop_black()

    for code in iter_codes(3):
        ds = dissect(code, 3)
        name = ''.join(str(x) for x in code)

        front_desc = calc_front_desc(ds)
        front = cave_front_dct['%s/z1' % front_desc] if front_desc is not None else None

        top = image2.stack((
            black_dct[ds[0]],
            cave_top_dct[ds[1]],
            cave_top_dct[ds[2]],
            ))

        clear = name in ('1111', '2222')
        BLOCK.new('cave_z1/c%s' % name) \
                .shape('empty' if clear else 'solid') \
                .top(top).front(front)

    # Ramps
    cave_bottom_dct = dict(iter_terrain_floor(collect_layers('gc')))
    do_cave_ramps(tiles, cave_bottom_dct)
    do_cave_entrance(tiles, cave_bottom_dct)

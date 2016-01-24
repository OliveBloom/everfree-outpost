from outpost_data.core import image2
from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.image2 import loader, Anim


# Throughout this module, we use "packed keys" that combine four small integers
# into a single number.  Typically the four integers represent the NW/NE/SE/SW
# corners of a block or tile.

def pack4(x, base):
    a, b, c, d = x
    return a + base * (b + base * (c + base * (d)))

def unpack4(n, base):
    a = n % base; n //= base
    b = n % base; n //= base
    c = n % base; n //= base
    d = n % base; n //= base
    return (a, b, c, d)


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
        desc = describe(tuple(w == v for w in x))
        areas.append(desc)
    return areas


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
        parts['inner/w/z%d' % z] = parts['outer/w/z%d' % z].modify(blank_left)
        parts['inner/e/z%d' % z] = parts['outer/e/z%d' % z].modify(blank_right)

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


# NB: Some stuff here may look backwards, but that's because this grid
# describes where the terrain *isn't*, or in other words, it describes the area
# where the underlying tile shows through.
TERRAIN_PARTS = (
       (None,       'inner/nw',     'inner/ne'),
       (None,       'inner/sw',     'inner/se'),
       ('outer/nw', 'half/n',       'outer/ne'),
       ('half/w',   'empty',        'half/e'),
       ('outer/sw', 'half/s',       'outer/se'),
       )

def chop_terrain(img):
    parts = img.chop_grid(TERRAIN_PARTS)
    parts['full'] = BLANK_TILE
    parts['cross/nw'] = image2.stack((parts['outer/ne'], parts['outer/sw']))
    parts['cross/ne'] = image2.stack((parts['outer/nw'], parts['outer/se']))
    return parts


def do_cave_walls(tiles):
    cave_img = tiles('lpc-cave-walls2.png')
    grass_img = tiles('lpc-base-tiles/grass.png')
    dirt_img = tiles('lpc-base-tiles/dirt.png')
    dirt2_img = tiles('lpc-base-tiles/dirt2.png')

    grass_base = grass_img.extract((1, 3))
    dirt_base = dirt_img.extract((1, 3))

    top_dct = chop_cave_top(cave_img)
    front_dct = chop_cave_front(cave_img)
    entrance_dct = chop_cave_entrance(cave_img)
    black_dct = chop_black()
    dirt2_dct = chop_terrain(dirt2_img)

    bb = BLOCK.prefixed('cave')

    for i in range(3 * 3 * 3 * 3):
        ds = dissect(unpack4(i, 3), 3)

        # Get top tile
        top = image2.stack((black_dct[ds[0]], top_dct[ds[1]], top_dct[ds[2]]))

        # Get front tile type
        need_w = False
        need_e = False
        front_desc = None
        for j in (1, 2):
            if ds[j] == 'outer/sw':
                front_desc = 'outer/w'
            elif ds[j] == 'outer/se':
                front_desc = 'outer/e'
            elif ds[j] == 'inner/nw' or ds[j] == 'cross/nw':
                need_w = True
            elif ds[j] == 'inner/ne' or ds[j] == 'cross/ne':
                need_e = True
            elif ds[j] == 'half/s':
                front_desc = 'center'
        if front_desc is None:
            if need_w and need_e:
                front_desc = 'center'
            elif need_w:
                front_desc = 'inner/w'
            elif need_e:
                front_desc = 'inner/e'

        # Get bottom tile type
        # Describe where grass should go (or, where dirt2 should *not* go)
        bottom_desc = describe(tuple(x == 1 for x in unpack4(i, 3)))

        # Build blocks
        nw, ne, se, sw = unpack4(i, 3)
        clear = nw == ne == se == sw and (nw == 1 or nw == 2)
        variants = [('', front_dct, clear)]
        if (ne, se) == (2, 1):
            variants.append(('entrance/left/', entrance_dct['left'], False))
        if (nw, sw) == (2, 1):
            variants.append(('entrance/right/', entrance_dct['right'], False))
        if (nw, ne, se, sw) == (2, 2, 1, 1):
            variants.append(('entrance/center/', entrance_dct['center'], True))

        for prefix, dct, clear in variants:
            bb_i = bb.prefixed(prefix + str(i))
            bb_i.new('z1').shape('empty' if clear else 'solid').top(top) \
                    .front(dct[front_desc + '/z1'] if front_desc is not None else None)
            bb_i.new('z0/grass').shape('floor' if clear else 'solid') \
                    .front(dct[front_desc + '/z0'] if front_desc is not None else None) \
                    .bottom(image2.stack((grass_base, dirt2_dct[bottom_desc])))
            bb_i.new('z0/dirt').shape('floor' if clear else 'solid') \
                    .front(dct[front_desc + '/z0'] if front_desc is not None else None) \
                    .bottom(image2.stack((dirt_base, dirt2_dct[bottom_desc])))

        if False:
            from PIL import Image
            im = Image.new('RGBA', (32, 96))
            im.paste(bb_i['z1'].top.raw().raw(), (0, 0))
            im.paste(bb_i['z0/grass'].bottom.raw().raw(), (0, 64))
            if front_desc:
                im.paste(bb_i['z1'].front.raw().raw(), (0, 32))
                x = bb_i['z0/grass'].front.raw().raw()
                im.paste(x, (0, 64), x)
            im.save('test-%s-%d,%d,%d,%d.png' % ((prefix.replace('/', '_'),) + unpack4(i, 3)))

def do_cave_top(tiles):
    img = tiles('lpc-cave-top.png')
    cross_img = tiles('lpc-cave-top.png')

    dct = chop_terrain(img)
    dct['cross/nw'] = cross_img.extract((0, 1))
    dct['cross/ne'] = cross_img.extract((0, 0))

    bb = BLOCK.prefixed('cave_top').shape('floor')

    for i in range(16):
        desc = describe(tuple(x == 0 for x in unpack4(i, 2)))
        bb.new(str(i)).bottom(dct[desc])
        bb[str(i)].bottom.raw().raw().save('test-%d-%s.png' % (i, desc.replace('/', '_')))

def init():
    tiles = loader('tiles', unit=TILE_SIZE)
    do_cave_walls(tiles)
    do_cave_top(tiles)



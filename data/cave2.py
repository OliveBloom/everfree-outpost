from outpost_data.core import image2
from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.image2 import loader, Anim
from outpost_data.core import structure
from outpost_data.outpost.lib import meshes


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

def calc_bottom_desc(i):
    return describe(tuple(x == 1 for x in unpack4(i, 3)))

class CaveWalls:
    def __init__(self, tiles):
        self.tiles = tiles

        self.grass_base = tiles('lpc-base-tiles/grass.png').extract((1, 3))
        self.dirt_base = tiles('lpc-base-tiles/dirt.png').extract((1, 3))
        self.dirt2_dct = chop_terrain(tiles('lpc-base-tiles/dirt2.png'))

        self.cave_img = tiles('lpc-cave-walls2.png')
        self.top_dct = chop_cave_top(self.cave_img)
        self.black_dct = chop_black()

    def make_top(self, ds):
        return image2.stack((
            self.black_dct[ds[0]],
            self.top_dct[ds[1]],
            self.top_dct[ds[2]],
            ))

    def make_wall_front_dict(self, z):
        front_dct = chop_cave_front(self.cave_img)
        result = {}

        for i in range(3 * 3 * 3 * 3):
            parts = unpack4(i, 3)
            ds = dissect(parts, 3)

            front_desc = calc_front_desc(ds)
            if front_desc is None:
                continue
            name = ''.join(str(x) for x in parts)
            result[name] = front_dct['%s/z%d' % (front_desc, z)]

        return result

    def do_cave_z1(self):
        front_dct = self.make_wall_front_dict(1)
        bb = BLOCK.prefixed('cave_z1')

        for i in range(3 * 3 * 3 * 3):
            parts = unpack4(i, 3)
            ds = dissect(parts, 3)
            name = ''.join(str(x) for x in parts)

            top = self.make_top(ds)
            front = front_dct.get(name)

            clear = name in ('1111', '2222')
            bb.new(name).shape('empty' if clear else 'solid').top(top).front(front)

    def do_cave_z0(self, code, imgname):
        bottom = chop_terrain(self.tiles(imgname))['empty']
        front_dct = self.make_wall_front_dict(0)
        bb = BLOCK.prefixed('terrain/%s' % (code * 4))

        for i in range(3 * 3 * 3 * 3):
            parts = unpack4(i, 3)
            ds = dissect(parts, 3)
            name = ''.join(str(x) for x in parts)

            front = front_dct.get(name)

            clear = name in ('1111', '2222')
            bb.new('c%s' % name).shape('floor' if clear else 'solid') \
                    .bottom(bottom).front(front)


    def do_block_z1(self, bb, name, clear, top, front):

        bb.new('z1').shape('empty' if clear else 'solid').top(top) \
                .front(front_dct[front_desc + '/z1'] if front_desc is not None else None)
        bb.new('z0/grass').shape('floor' if clear else 'solid') \
                .front(front_dct[front_desc + '/z0'] if front_desc is not None else None) \
                .bottom(image2.stack((self.grass_base, self.dirt2_dct[bottom_desc])))
        bb.new('z0/dirt').shape('floor' if clear else 'solid') \
                .front(front_dct[front_desc + '/z0'] if front_desc is not None else None) \
                .bottom(image2.stack((self.dirt_base, self.dirt2_dct[bottom_desc])))

    def do_block(self, bb, clear, front_dct, top, front_desc, bottom_desc):
        bb.new('z1').shape('empty' if clear else 'solid').top(top) \
                .front(front_dct[front_desc + '/z1'] if front_desc is not None else None)
        bb.new('z0/grass').shape('floor' if clear else 'solid') \
                .front(front_dct[front_desc + '/z0'] if front_desc is not None else None) \
                .bottom(image2.stack((self.grass_base, self.dirt2_dct[bottom_desc])))
        bb.new('z0/dirt').shape('floor' if clear else 'solid') \
                .front(front_dct[front_desc + '/z0'] if front_desc is not None else None) \
                .bottom(image2.stack((self.dirt_base, self.dirt2_dct[bottom_desc])))

        if False:
            from PIL import Image
            im = Image.new('RGBA', (32, 96))
            im.paste(bb['z1'].top.raw().raw(), (0, 0))
            im.paste(bb['z0/grass'].bottom.raw().raw(), (0, 64))
            if front_desc:
                im.paste(bb['z1'].front.raw().raw(), (0, 32))
                x = bb['z0/grass'].front.raw().raw()
                im.paste(x, (0, 64), x)
            im.save('test-%s.png' % bb._full_prefix.replace('/', '_'))

    def do_cave_walls(self):
        front_dct = chop_cave_front(self.cave_img)

        bb = BLOCK.prefixed('cave')
        for i in range(3 * 3 * 3 * 3):
            ds = dissect(unpack4(i, 3), 3)

            top = self.make_top(ds)
            front_desc = calc_front_desc(ds)
            bottom_desc = calc_bottom_desc(i)

            # Build blocks
            nw, ne, se, sw = unpack4(i, 3)
            clear = nw == ne == se == sw and (nw == 1 or nw == 2)

            self.do_block(bb.prefixed(str(i)), clear, front_dct,
                    top, front_desc, bottom_desc)

    def do_cave_entrance(self):
        entrance_dct = chop_cave_entrance(self.cave_img)

        bb = BLOCK.prefixed('cave/entrance')
        for i in range(3 * 3 * 3 * 3):
            ds = dissect(unpack4(i, 3), 3)

            # Build blocks
            nw, ne, se, sw = unpack4(i, 3)

            east_ok = (ne == 2 and se == 1)
            west_ok = (nw == 2 and sw == 1)
            if not east_ok and not west_ok:
                continue

            top = self.make_top(ds)
            front_desc = calc_front_desc(ds)
            bottom_desc = calc_bottom_desc(i)

            if east_ok:
                self.do_block(bb.prefixed('left/%d' % i), False, entrance_dct['left'],
                        top, front_desc, bottom_desc)
            if west_ok:
                self.do_block(bb.prefixed('right/%d' % i), False, entrance_dct['right'],
                        top, front_desc, bottom_desc)
            if east_ok and west_ok:
                self.do_block(bb.prefixed('center/%d' % i), True, entrance_dct['center'],
                        top, front_desc, bottom_desc)

    def do_natural_ramp(self):
        ramp_img = self.tiles('outdoor-ramps.png').extract((0, 0), (3, 5))
        ramp_top_dct = chop_ramp_top(self.cave_img, ramp_img)
        ramp_front_dct = chop_ramp_front(self.cave_img, ramp_img)

        bb = BLOCK.prefixed('natural_ramp')
        for i in range(3 * 3 * 3 * 3):
            nw, ne, se, sw = unpack4(i, 3)

            left_ok = (ne == se == 1 and nw != 1)
            right_ok = (nw == sw == 1 and ne != 1)
            back_ok = (sw == se == 1 and nw != 1 and ne != 1)

            if not left_ok and not right_ok and not back_ok:
                continue

            ds = dissect(unpack4(i, 3), 3)

            top = image2.stack((
                self.black_dct[ds[0]],
                ramp_top_dct[ds[1]],
                self.top_dct[ds[2]],
                ))
            front_desc = calc_front_desc(ds) or 'empty'
            bottom_desc = calc_bottom_desc(i)

            # Build blocks
            if left_ok:
                self.do_block(bb.prefixed('left/%d' % i), False, ramp_front_dct['left'],
                        top, front_desc, bottom_desc)
            if right_ok:
                self.do_block(bb.prefixed('right/%d' % i), False, ramp_front_dct['right'],
                        top, front_desc, bottom_desc)
            if back_ok:
                bb.new('back/%d' % i) \
                        .shape('solid') \
                        .top(top)

        bb.new('top') \
                .shape('floor') \
                .bottom(ramp_img.extract((0, 4)))

        bb.new('ramp/z0/grass') \
                .shape('ramp_n') \
                .bottom(image2.stack((self.grass_base, ramp_img.extract((1, 4))))) \
                .back(ramp_img.extract((1, 3)))

        bb.new('ramp/z0/dirt') \
                .shape('ramp_n') \
                .bottom(image2.stack((self.dirt_base, ramp_img.extract((1, 4))))) \
                .back(ramp_img.extract((1, 3)))

        bb.new('ramp/z1') \
                .shape('ramp_n') \
                .bottom(ramp_img.extract((1, 2))) \
                .back(ramp_img.extract((1, 1)))

def do_cave_top(tiles, code, imgname):
    img = tiles('%s.png' % imgname)
    cross_img = tiles('%s-cross.png' % imgname)

    dct = chop_terrain(img)
    dct['cross/nw'] = cross_img.extract((0, 1))
    dct['cross/ne'] = cross_img.extract((0, 0))

    bb = BLOCK.prefixed('terrain/%s' % (code * 4)).shape('floor')

    for i in range(16):
        digits = unpack4(i, 2)
        desc = describe(digits)
        name = 'e' + ''.join(str(x) for x in digits)
        bb.new(name).bottom(dct[desc])

    bb = BLOCK.new('terrain/%s' % (code * 4)).shape('floor').bottom(dct['empty'])


def do_cave_junk(img):
    sb = STRUCTURE.prefixed('cave_junk') \
            .mesh(meshes.front(1, 1, 1)) \
            .shape(structure.solid(1, 1, 1)) \
            .layer(1)
    for i in range(3):
        sb.new(str(i)).image(img.extract((i, 0)))


def init():
    tiles = loader('tiles', unit=TILE_SIZE)
    structures = loader('structures', unit=TILE_SIZE)

    do_cave_top(tiles, 'm', 'lpc-cave-top')
    do_cave_top(tiles, 'g', 'cave-top-grass')

    cw = CaveWalls(tiles)
    cw.do_cave_walls()
    cw.do_cave_entrance()
    cw.do_natural_ramp()

    cw.do_cave_z0('g', 'cave-top-grass.png')
    cw.do_cave_z1()

    do_cave_junk(structures('cave-junk.png'))

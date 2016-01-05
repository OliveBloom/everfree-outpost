from collections import namedtuple

from outpost_server.core import use, util
from outpost_server.core.data import DATA
from outpost_server.core.types import V3

from outpost_server.outpost.lib import mallet, structure_items

Sides = namedtuple('Sides', ('n', 's', 'w', 'e'))
ZERO_SIDES = Sides(0, 0, 0, 0)


class Category:
    def __init__(self, layer, side_map, choices):
        self._layer = layer
        # Default side values to use when adding new groups
        self._side_map = side_map
        self._choices = choices

        # Map every recognized template to its Sides
        self._sides = {}

    def add_group(self, basename, side_map=None):
        if side_map is None:
            side_map = self._side_map

        for k,v in side_map.items():
            template = DATA.get_template(basename + '/' + k)
            print(basename, k, template, v)
            if template is None:
                continue
            self._sides[template] = v

    def inner_sides(self, plane, pos):
        s = plane.find_structure_at_point_layer(pos, self._layer)
        if s is None:
            return ZERO_SIDES
        sides = self._sides.get(s.template())
        if sides is None:
            return ZERO_SIDES
        return sides

    def outer_sides(self, plane, pos):
        n = self.inner_sides(plane, pos - V3(0, 1, 0)).s
        s = self.inner_sides(plane, pos + V3(0, 1, 0)).n
        w = self.inner_sides(plane, pos - V3(1, 0, 0)).e
        e = self.inner_sides(plane, pos + V3(1, 0, 0)).w
        return Sides(n, s, w, e)

    def choose_best(self, basename, evaluate):
        best_template = self.get_default(basename)
        best_value = None
        for name in self._choices:
            template = DATA.get_template(basename + '/' + name)
            if template is None:
                continue
            sides = self._sides.get(template)
            if sides is None:
                continue
            value = evaluate(sides)

            # NB: strict > here, not >=.  This way, if there are multiple
            # choices that maximize value, we always take the first.
            if best_value is None or value > best_value:
                best_template = template
                best_value = value
        return best_template

    def get_default(self, basename):
        for name in self._choices:
            template = DATA.get_template(basename + '/' + name)
            if template is not None:
                return template
        return None

def side_sum(s, f):
    def g(t):
        return f(s.n, t.n) + f(s.s, t.s) + f(s.w, t.w) + f(s.e, t.e)
    return g

def count_equal(s):
    def g(t):
        return int(s.n == t.n) + int(s.s == t.s) + int(s.w == t.w) + int(s.e == t.e)
    return g

def _register(item, basename, cat, choose):
    if basename is None:
        basename = item
    item = DATA.item(item)

    cat.add_group(basename)

    @use.item(item)
    def use_item(e, args):
        pos = util.hit_tile(e)
        template = choose(basename, e.plane(), pos)
        structure_items.place(e, item, template)

    @use.structure(cat.get_default(basename))
    def use_structure(e, s, args):
        structure_items.take(e, s, item)


# Floor category

# 0: no floor, 1: floor on -x/-y half, 2: floor on +x/+y half, 3: both
FLOOR_SIDES = {
        'center/v0':        Sides(3, 3, 3, 3),
        'center/v1':        Sides(3, 3, 3, 3),
        'center/v2':        Sides(3, 3, 3, 3),
        'center/v3':        Sides(3, 3, 3, 3),
        'edge/n':           Sides(0, 3, 2, 2),
        'edge/s':           Sides(3, 0, 1, 1),
        'edge/w':           Sides(2, 2, 0, 3),
        'edge/e':           Sides(1, 1, 3, 0),
        'corner/outer/nw':  Sides(0, 2, 0, 2),
        'corner/outer/sw':  Sides(2, 0, 0, 1),
        'corner/outer/ne':  Sides(0, 1, 2, 0),
        'corner/outer/se':  Sides(1, 0, 1, 0),
        'corner/inner/nw':  Sides(2, 3, 2, 3),
        'corner/inner/ne':  Sides(1, 3, 3, 2),
        'corner/inner/se':  Sides(3, 1, 3, 1),
        'corner/inner/sw':  Sides(3, 2, 1, 3),
    }

FLOOR_ORDER = (
        'center/v0',
        'edge/n',
        'edge/s',
        'edge/w',
        'edge/e',
        'corner/outer/nw',
        'corner/outer/sw',
        'corner/outer/ne',
        'corner/outer/se',
        'corner/inner/nw',
        'corner/inner/ne',
        'corner/inner/se',
        'corner/inner/sw',
    )

FLOOR_CAT = Category(0, FLOOR_SIDES, FLOOR_ORDER)

def compare_floor_sides(a, b):
    # Value matching up non-empty sides slightly more than matching up empty
    # sides.  This makes placing a floor on an inner corner prefer the actual
    # inner corner tile over an edge tile.  (Corner matches 3, 3, 1/2, and
    # mismatches 0; edge matches 3, 0, 1/2, and mismatches 3.)
    if a == b:
        if a == 0:
            return 10
        else:
            return 11
    else:
        return 0

def choose_floor_variant(basename, plane, pos):
    sides = FLOOR_CAT.outer_sides(plane, pos)
    if not any(x in (1, 2) for x in sides):
        # When there are no interesting sides next to this position (all are
        # either solid or empty), just place the default (center) variant.
        # This lets the player easily control when autorotation kicks in - they
        # can fill the middle of their area with center tiles (no autorotate),
        # then place and mallet a single edge tile to make autorotate kick in
        # as they fill in the rest of the border from there.
        return FLOOR_CAT.get_default(basename)
    else:
        return FLOOR_CAT.choose_best(basename,
                side_sum(sides, compare_floor_sides))

def register_floor(item, basename=None):
    _register(item, basename, FLOOR_CAT, choose_floor_variant)


# Wall category

# 0: nothing, 1: wall
WALL_SIDES = {
        'edge/horiz':       Sides(0, 0, 1, 1),
        'edge/vert':        Sides(1, 1, 0, 0),
        'corner/nw':        Sides(0, 1, 0, 1),
        'corner/ne':        Sides(0, 1, 1, 0),
        'corner/sw':        Sides(1, 0, 0, 1),
        'corner/se':        Sides(1, 0, 1, 0),
        'tee/n':            Sides(1, 0, 1, 1),
        'tee/e':            Sides(1, 1, 0, 1),
        'tee/s':            Sides(0, 1, 1, 1),
        'tee/w':            Sides(1, 1, 1, 0),
        'cross':            Sides(1, 1, 1, 1),
        'door/closed':      Sides(0, 0, 1, 1),
    }

WALL_ORDER = (
        'edge/horiz',
        'edge/vert',
        'corner/nw',
        'corner/ne',
        'corner/sw',
        'corner/se',
        'tee/n',
        'tee/e',
        'tee/s',
        'tee/w',
        'cross',
        'door/closed',
    )

WALL_CAT = Category(1, WALL_SIDES, WALL_ORDER)

def compare_wall_to_floor(f, w):
    # If the floor has an edge here, then the wall should extend in this
    # direction.
    return (f in (1, 2)) == (w == 1)

def choose_wall_variant(basename, plane, pos):
    floor_sides = FLOOR_CAT.inner_sides(plane, pos)
    best = WALL_CAT.choose_best(basename,
            side_sum(floor_sides, compare_wall_to_floor))
    return best or DATA.template(basename + '/edge/horiz')

def register_wall(item, basename=None):
    _register(item, basename, WALL_CAT, choose_wall_variant)



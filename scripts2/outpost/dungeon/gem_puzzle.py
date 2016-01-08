from outpost_server.core import use, util
from outpost_server.core.data import DATA
from outpost_server.core.engine import StructureProxy

from outpost_server.outpost.lib import door

NORMAL_EMPTY = DATA.template('dungeon/gem_slot/normal/empty')
COLORS = ('red', 'orange', 'yellow', 'green', 'blue', 'purple')
COLOR_IDX = {c: i for i,c in enumerate(COLORS)}

def register_color(color):
    gem = DATA.item('gem/' + color)
    normal_slot = DATA.template('dungeon/gem_slot/normal/' + color)
    fixed_slot = DATA.template('dungeon/gem_slot/fixed/' + color)

    @use.structure(fixed_slot)
    def use_fixed(e, s, args):
        e.controller().send_message("You can't remove this gem.")

    @use.structure(normal_slot)
    def use_normal(e, s, args):
        _, _, color = s.template().name.rpartition('/')
        if color == 'empty':
            return

        if e.inv('main').count_space(gem) == 0:
            return
        s.replace(NORMAL_EMPTY)
        e.inv('main').bulk_add(gem, 1)

        update_puzzle(s, None)

    @use.item(gem)
    def use_gem(e, args):
        s = util.hit_structure(e)
        if s is None or s.template() is not NORMAL_EMPTY:
            return
        e.inv('main').bulk_remove(gem, 1)
        s.replace(normal_slot)

        update_puzzle(s, COLOR_IDX[color])

def update_puzzle(s, color):
    puzzle_id = s.extra()['puzzle_id']
    slot = s.extra()['puzzle_slot']
    puzzle = s.plane().extra()['puzzles'][puzzle_id]

    slots = puzzle['slots']
    slots[slot] = color

    if all(c is not None for c in slots):
        open_door = check_slots(slots)
    else:
        open_door = False

    s_door = s.engine.stable_structure(puzzle['door'])
    if open_door:
        door.open(s_door)
    else:
        door.close(s_door)

def check_slots(slots):
    for i, c in enumerate(slots):
        if i % 2 == 0:
            # Slots 0, 2, ... should contain red (0), yellow (2), or blue (4)
            if c % 2 != 0:
                return False
        else:
            # Slots 1, 3, ... should contain the secondary color that results
            # from blending the two primaries
            if c % 2 != 1:
                return False
            a = (c - 1 + 6) % 6
            b = (c + 1) % 6

            l = slots[i - 1]
            r = slots[i + 1]

            if not ((l == a and r == b) or (l == b and r == a)):
                return False
    return True

for c in COLORS:
    register_color(c)

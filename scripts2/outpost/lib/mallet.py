from outpost_server.core import alias, util
from outpost_server.core.data import DATA

_NEXT_VARIANT = {}

def register(base, names):
    prev = DATA.template(base + names[-1])
    for n in names:
        cur = DATA.template(base + n)
        _NEXT_VARIANT[prev] = cur
        prev = cur

    first = DATA.template(base + names[0])
    for n in names[1:]:
        alias.register_template(base + n, first)

def use(e, args=None):
    s = util.hit_structure(e)
    if s is None:
        return

    old_template = s.template()
    new_template = _NEXT_VARIANT.get(old_template)
    if new_template is None:
        return
    s.replace(new_template)


TERRAIN_VARIANTS = (
    'center/v0',
    'edge/n', 'corner/outer/ne', 'edge/e', 'corner/outer/se',
    'edge/s', 'corner/outer/sw', 'edge/w', 'corner/outer/nw',
    'corner/inner/nw', 'corner/inner/ne', 'corner/inner/se', 'corner/inner/sw',
)

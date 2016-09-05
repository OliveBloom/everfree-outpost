from outpost_server.core import use, util
from outpost_server.core.data import DATA

from outpost_server.outpost.lib import tool as tool_, ward

LAYER_ATTACH = 2

def place(e, item, template=None, ignore_ward=False):
    pos = util.hit_tile(e)
    if not ignore_ward:
        ward.check(e, pos) 

    item = DATA.item(item)
    template = DATA.template(template if template is not None else item.name)

    if template.layer == LAYER_ATTACH:
        print(template, _DEFAULT_BASE.get(template))
        if check_attachment(template, e.plane(), pos):
            pass
        elif template in _DEFAULT_BASE:
            s = e.plane().create_structure(pos, _DEFAULT_BASE[template])
            print(s, s.template(), s.pos())
        else:
            raise RuntimeError('invalid base for attachment')

    if e.inv('main').count(item) == 0:
        raise RuntimeError('missing item in inventory')

    # If create_structure raises an exception, the item will not be used up.
    s = e.plane().create_structure(pos, template)
    e.inv('main').bulk_remove(item, 1)

    return s

def take(e, s, item, ignore_ward=False):
    if not ignore_ward:
        ward.check(e, s.pos())

    item = DATA.item(item)
    template = s.template()
    pos = s.pos()

    if e.inv('main').count_space(item) == 0:
        raise RuntimeError('no space for item in inventory')
    s.destroy()
    e.inv('main').bulk_add(item, 1)

    if template.layer == LAYER_ATTACH and template in _DEFAULT_BASE:
        base = e.plane().find_structure_at_point_layer(pos, 1)
        if base is not None and base.template() == _DEFAULT_BASE[template]:
            base.destroy()

def register(item, template=None, tool=None):
    item = DATA.item(item)
    template = DATA.template(template if template is not None else item.name)

    use.item(item)(lambda e, args: place(e, item, template))
    if tool is None:
        use.structure(template)(lambda e, s, args: take(e, s, item))
    else:
        tool_.handler(tool, template)(lambda e, s, args: take(e, s, item))

_ATTACH_MAP = {}
_BASE_MAP = {}
_DEFAULT_BASE = {}

def register_attachment(template, cls, default_base=None):
    template = DATA.template(template)
    _ATTACH_MAP[template] = cls
    if default_base is not None:
        _DEFAULT_BASE[template] = DATA.template(default_base)

def register_base(template, cls):
    template = DATA.template(template)
    _BASE_MAP.setdefault(cls, set()).add(template)

def check_attachment(template, plane, pos):
    """Check if `template` can legally be attached to whatever structure
    currently exists at `pos`.  Returns `False` if placement should be blocked,
    or `True` if the normal placement algorithm should be used."""
    if template not in _ATTACH_MAP:
        return True

    s = plane.find_structure_at_point_layer(pos, 1)
    if s is None:
        return False

    cls = _ATTACH_MAP[template]
    bases = _BASE_MAP.get(cls)
    return bases is not None and s.template() in bases

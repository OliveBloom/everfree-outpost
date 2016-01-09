from outpost_server.core.types import V3
from outpost_server.core.engine import EntityProxy, StructureProxy

RADIUS = 16
SPACING = 48

class Wards(object):
    def __init__(self, plane):
        self.plane = plane

    def get_info(self):
        info = self.plane.extra().get('ward_info')
        if info is None:
            info = self.plane.extra().setdefault('ward_info', {
                'server': {
                    'pos': V3(0, 0, 0),
                    'name': 'the server',
                },
            })
            info = self.plane.extra()['ward_info']
        return info

    def has_ward(self, key):
        return (key in self.get_info())

    def add_ward(self, key, pos, name):
        dct = {
                'pos': pos,
                'name': name,
                }
        self.get_info()[key] = dct

    def remove_ward(self, key):
        del self.get_info()[key]

    def find_wards(self, pos, radius):
        keys = []
        for k,v in self.get_info().items():
            d = v['pos'] - pos
            dist = max(abs(x) for x in (d.x, d.y, d.z))
            if dist <= radius:
                keys.append(k)
        return keys

    def get_name(self, key):
        return self.get_info()[key]['name']


class Permissions(object):
    def __init__(self, eng):
        self.eng = eng

    def get_perms(self):
        return self.eng.world_extra().setdefault('ward_perms', {})

    def add_perm(self, key, name):
        perms = self.get_perms()
        if key not in perms:
            perms[key] = {}
        perms[key][name] = True

    def remove_perm(self, key, name):
        perms = self.get_perms()
        del perms[key][name]
        if len(perms[key]) == 0:
            del perms[key]

    def has_perm(self, key, name):
        """Check if `key` has granted permission to `name`."""
        perms = self.get_perms()
        return (key in perms and name in perms[key])


def get_key(x):
    if isinstance(x, EntityProxy):
        return str(x.stable_id().raw)
    elif isinstance(x, StructureProxy):
        return str(x.extra()['owner'].raw)
    else:
        assert False, 'expected entity or structure'


def permit(e, name):
    key = get_key(e)
    p = Permissions(e.engine)
    p.add_perm(key, name)

def revoke(e, name):
    key = get_key(e)
    p = Permissions(e.engine)
    if p.has_perm(key, name):
        p.remove_perm(key, name)
        return True
    else:
        return False

def can_add(e, pos):
    """Check whether it's legal for `e` to place a ward at `pos`.  Returns
    (True, None) on success or (False, msg) on failure.
    """
    key = get_key(e)
    w = Wards(e.plane())
    if w.has_ward(key):
        return False, 'You may only place one ward at a time'
    p = Permissions(e.engine)
    name = e.controller().name()

    for k in w.find_wards(pos, SPACING):
        if not p.has_perm(k, name):
            return False, 'This area is too close to land belonging to %s' % w.get_name(k)

    return True, None

def can_remove(e, s):
    """Check whether it's legal for `e` to remove ward `s`."""
    if s.extra()['owner'] != e.stable_id():
        key = get_key(s)
        w = Wards(e.plane())
        return False, 'This ward belongs to %s' % w.get_name(key)
    else:
        return True, None

def can_act(e, pos):
    """Check whether it's legal for `e` to take an action at `pos`."""
    w = Wards(e.plane())
    p = Permissions(e.engine)
    name = e.controller().name()

    for k in w.find_wards(pos, RADIUS):
        if not p.has_perm(k, name):
            return False, 'This area belongs to %s' % w.get_name(k)

    return True, None

def add(e, pos, s):
    key = get_key(e)
    w = Wards(e.plane())
    w.add_ward(key, pos, e.controller().name())
    s.extra()['owner'] = e.stable_id()

def remove(s):
    key = get_key(s)
    w = Wards(s.plane())
    w.remove_ward(key)

def check(e, pos):
    """Ensure that it's legal for `e` to take an action at `pos`.  If it's not,
    send the user a message and raise an exception."""
    ok, msg = can_act(e, pos)
    if not ok:
        e.controller().send_message(msg)
        if not e.controller().is_superuser():
            raise RuntimeError('ward check failed: %r' % msg)

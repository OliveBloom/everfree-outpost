from outpost_server.core.types import V3

RADIUS = 16
SPACING = 48

def get_key(e):
    return str(e.stable_id().raw)

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

    def has_ward(self, e):
        key = get_key(e)
        return (key in self.get_info())

    def add_ward(self, e, pos):
        key = get_key(e)
        dct = {
                'pos': pos,
                'name': e.controller().name(),
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

    def add_perm(self, e, name):
        key = get_key(e)
        perms = self.get_perms()
        if key not in perms:
            perms[key] = {}
        perms[key][name] = True

    def remove_perm(self, e, name):
        key = get_key(e)
        perms = self.get_perms()
        del perms[key][name]
        if len(perms[key]) == 0:
            del perms[key]

    def has_perm(self, key, name):
        perms = self.get_perms()
        return (key in perms and name in perms[key])


def permit(e, name):
    p = Permissions(e.engine)
    p.add_perm(e, name)

def revoke(e, name):
    p = Permissions(e.engine)
    if p.has_perm(get_key(e), name):
        p.remove_perm(e, name)
        return True
    else:
        return False

def can_add(e, pos):
    """Check whether it's legal for `e` to place a ward at `pos`.  Returns
    (True, None) on success or (False, msg) on failure.
    """
    w = Wards(e.plane())
    if w.has_ward(e):
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
        key = str(s.extra()['owner'].raw)
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
    w = Wards(e.plane())
    w.add_ward(e, pos)
    s.extra()['owner'] = e.stable_id()

def remove(s):
    w = Wards(s.plane())
    key = str(s.extra()['owner'].raw)
    w.remove_ward(key)

def check(e, pos):
    """Ensure that it's legal for `e` to take an action at `pos`.  If it's not,
    send the user a message and raise an exception."""
    ok, msg = can_act(e, pos)
    if not ok:
        e.controller().send_message(msg)
        raise RuntimeError('ward check failed: %r' % msg)

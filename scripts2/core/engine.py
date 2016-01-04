from outpost_server.core.extra import ExtraHashProxy
from outpost_server.core.types import *

def check_type(obj, ty):
    if not isinstance(obj, ty):
        raise ValueError('expected %r, but got %r' % (ty, type(obj)))

class EngineProxy(object):
    def __init__(self, eng):
        self._eng = eng

    def __repr__(self):
        return '<engine at 0x%x>' % id(self._eng)

    def num_clients(self):
        return self._eng.messages_clients_len()


class ClientProxy(object):
    def __init__(self, eng, id):
        self._eng = eng
        self._engine = None
        assert isinstance(id, ClientId)
        self.id = id

    def __hash__(self):
        return hash(self.id)

    def __repr__(self):
        return '<client #%d>' % self.id.raw

    @property
    def engine(self):
        if self._engine is None:
            self._engine = EngineProxy(self._eng)
        return self._engine

    def send_message(self, msg):
        self._eng.messages_send_chat_update(self.id, '***\t' + msg)

    def pawn(self):
        eid = self._eng.world_client_pawn_id(self.id)
        if eid is not None:
            return EntityProxy(self._eng, eid)
        else:
            return None


class EntityProxy(object):
    def __init__(self, eng, id):
        self._eng = eng
        self._engine = None
        assert isinstance(id, EntityId)
        self.id = id

    def __hash__(self):
        return hash(self.id)

    def __repr__(self):
        return '<entity #%d>' % self.id.raw

    @property
    def engine(self):
        if self._engine is None:
            self._engine = EngineProxy(self._eng)
        return self._engine

    def pos(self):
        return self._eng.world_entity_pos(self.id)

    def plane(self):
        pid = self._eng.world_entity_plane_id(self.id)
        return PlaneProxy(self._eng, pid)

    def teleport(self, pos):
        self._eng.world_entity_teleport(self.id, pos)

    def teleport_plane(self, stable_pid, pos):
        check_type(stable_pid, StablePlaneId)
        self._eng.world_entity_teleport_stable_plane(self.id, stable_pid, pos)

    def extra(self):
        return ExtraHashProxy(self._eng.world_entity_extra(self.id))


class PlaneProxy(object):
    def __init__(self, eng, id):
        self._eng = eng
        self._engine = None
        assert isinstance(id, PlaneId)
        self.id = id

    def __hash__(self):
        return hash(self.id)

    def __repr__(self):
        return '<plane #%d>' % self.id.raw

    @property
    def engine(self):
        if self._engine is None:
            self._engine = EngineProxy(self._eng)
        return self._engine

    @property
    def name(self):
        return self._eng.world_plane_name(self.id)

    def stable_id(self):
        return self._eng.world_plane_stable_id(self.id)

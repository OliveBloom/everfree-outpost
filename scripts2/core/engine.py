from outpost_server.core.data import ItemProxy, TemplateProxy
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


class ObjectProxy(object):
    def __init__(self, eng, id):
        self._eng = eng
        self._engine = None
        assert isinstance(id, type(self).ID_TYPE)
        self.id = id

    def __hash__(self):
        return hash(self.id)

    def __repr__(self):
        return '<%s #%d>' % (type(self).__name__, self.id.raw)

    @property
    def engine(self):
        if self._engine is None:
            self._engine = EngineProxy(self._eng)
        return self._engine


class ClientProxy(ObjectProxy):
    ID_TYPE = ClientId

    def send_message(self, msg):
        self._eng.messages_send_chat_update(self.id, '***\t' + msg)

    def pawn(self):
        eid = self._eng.world_client_pawn_id(self.id)
        if eid is not None:
            return EntityProxy(self._eng, eid)
        else:
            return None

    def open_crafting(self, s, i):
        self._eng.logic_open_crafting(self.id, s.id, i.id)


class EntityProxy(ObjectProxy):
    ID_TYPE = EntityId

    def pos(self):
        return self._eng.world_entity_pos(self.id)

    def plane(self):
        pid = self._eng.world_entity_plane_id(self.id)
        return PlaneProxy(self._eng, pid)

    def facing(self):
        return self._eng.world_entity_facing(self.id)

    def teleport(self, pos):
        self._eng.world_entity_teleport(self.id, pos)

    def teleport_plane(self, stable_pid, pos):
        check_type(stable_pid, StablePlaneId)
        self._eng.world_entity_teleport_stable_plane(self.id, stable_pid.raw, pos)

    def extra(self):
        return ExtraHashProxy(self._eng.world_entity_extra(self.id))

    def inv(self, key):
        inv = self.extra().get('inv')
        if inv is None:
            return None
        iid = inv.get(key)
        if iid is None:
            return None
        return InventoryProxy(self._eng, iid)


class InventoryProxy(ObjectProxy):
    ID_TYPE = InventoryId

    def count(self, item):
        if isinstance(item, ItemProxy):
            item = item.id
        return self._eng.world_inventory_count(self.id, item)

    def count_space(self, item):
        if isinstance(item, ItemProxy):
            item = item.id
        return self._eng.world_inventory_count_space(self.id, item)

    def bulk_add(self, item, count):
        return self._eng.world_inventory_bulk_add(self.id, item.id, count)

    def bulk_remove(self, item, count):
        return self._eng.world_inventory_bulk_remove(self.id, item.id, count)


class PlaneProxy(ObjectProxy):
    ID_TYPE = PlaneId

    @property
    def name(self):
        return self._eng.world_plane_name(self.id)

    def stable_id(self):
        return self._eng.world_plane_stable_id(self.id)

    def find_structure_at_point(self, pos):
        opt_sid = self._eng.world_structure_find_at_point(self.id, pos)
        if opt_sid is not None:
            return StructureProxy(self._eng, opt_sid)
        else:
            return None

    def create_structure(self, pos, template):
        sid = self._eng.world_structure_create(self.id, pos, template.id)
        return StructureProxy(self._eng, sid)


class StructureProxy(ObjectProxy):
    ID_TYPE = StructureId

    def pos(self):
        return self._eng.world_structure_pos(self.id)

    def plane(self):
        pid = self._eng.world_structure_plane_id(self.id)
        return PlaneProxy(self._eng, pid)

    def template(self):
        id = self._eng.world_structure_template_id(self.id)
        return TemplateProxy.by_id(id)


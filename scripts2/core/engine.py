from outpost_server.core.data import DATA, BlockProxy, TemplateProxy
from outpost_server.core.extra import ExtraHashProxy
from outpost_server.core.types import *

def check_type(obj, ty):
    if not isinstance(obj, ty):
        raise ValueError('expected %r, but got %r' % (ty, type(obj)))


def _inv(self, key='main'):
    inv = self.extra().get('inv')
    if inv is None:
        return None
    iid = inv.get(key)
    if iid is None:
        return None
    return InventoryProxy(self._eng, iid)

def _create_inv(self, key, size):
    i = self.engine.create_inventory(size)
    i.attach(self.id)

    inv = self.extra().setdefault('inv', {})
    assert key not in inv, 'inventory %r already exists' % key
    inv[key] = i.id

    return i


class EngineProxy(object):
    def __init__(self, eng):
        self._eng = eng

    def __repr__(self):
        return '<engine at 0x%x>' % id(self._eng)

    def now(self):
        return self._eng.now()

    def num_clients(self):
        return self._eng.messages_clients_len()

    def world_extra(self):
        return ExtraHashProxy(self._eng.world_extra())


    def create_inventory(self, size):
        iid = self._eng.world_inventory_create(size)
        return InventoryProxy(self._eng, iid)

    def create_plane(self, name):
        pid = self._eng.world_plane_create(name)
        return PlaneProxy(self._eng, pid)


    def client_by_name(self, name):
        cid = self._eng.messages_client_by_name(name)
        return ClientProxy(self._eng, cid) if cid is not None else None

    def stable_entity(self, stable_eid):
        eid = self._eng.world_entity_transient_id(stable_eid)
        return EntityProxy(self._eng, eid) if eid is not None else None

    def stable_plane(self, stable_pid):
        pid = self._eng.world_plane_transient_id(stable_pid)
        return PlaneProxy(self._eng, pid) if eid is not None else None

    def stable_structure(self, stable_sid):
        sid = self._eng.world_structure_transient_id(stable_sid)
        return StructureProxy(self._eng, sid) if sid is not None else None


    def schedule_timer(self, when, userdata):
        return self._eng.timer_schedule(when, userdata)

    def cancel_timer(self, cookie):
        self._eng.timer_cancel(cookie)


    def get_object(self, id):
        if type(id) is StructureId:
            return self.get_structure(id)
        else:
            raise TypeError('unsupported ID type: %s' % type(id).__name__)

    def get_structure(self, sid):
        if self._eng.world_structure_check(sid):
            return StructureProxy(self._eng, sid)
        else:
            return None


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

    def name(self):
        return self._eng.world_client_name(self.id)

    def send_message(self, msg):
        self._eng.messages_send_chat_update(self.id, '***\t' + msg)

    def pawn(self):
        eid = self._eng.world_client_pawn_id(self.id)
        if eid is not None:
            return EntityProxy(self._eng, eid)
        else:
            return None

    def open_container(self, i1, i2):
        self._eng.logic_open_container(self.id, i1.id, i2.id)

    def open_crafting(self, s, i):
        self._eng.logic_open_crafting(self.id, s.id, i.id)

    def get_interact_args(self, dialog_id, args):
        self._eng.messages_send_get_interact_args(self.id, dialog_id, args)

    def get_use_item_args(self, item, dialog_id, args):
        item = DATA.item_id(ability)
        self._eng.messages_send_get_use_item_args(self.id, item, dialog_id, args)

    def get_use_ability_args(self, ability, dialog_id, args):
        ability = DATA.item_id(ability)
        self._eng.messages_send_get_use_ability_args(self.id, ability, dialog_id, args)

    def extra(self):
        return ExtraHashProxy(self._eng.world_client_extra(self.id))

    def is_superuser(self):
        return bool(self.extra().get('superuser'))

    def set_main_inventories(self, i_item, i_ability):
        self._eng.logic_set_main_inventories(self.id, i_item.id, i_ability.id)

    inv = _inv
    create_inv = _create_inv


class EntityProxy(ObjectProxy):
    ID_TYPE = EntityId

    def stable_id(self):
        return self._eng.world_entity_stable_id(self.id)

    def pos(self):
        return self._eng.world_entity_pos(self.id)

    def plane(self):
        pid = self._eng.world_entity_plane_id(self.id)
        return PlaneProxy(self._eng, pid)

    def facing(self):
        return self._eng.world_entity_facing(self.id)

    def appearance(self):
        return self._eng.world_entity_appearance(self.id)

    def set_appearance(self, appearance):
        self._eng.world_entity_set_appearance(self.id, appearance)

    def controller(self):
        cid = self._eng.world_entity_controller(self.id)
        if cid is None:
            return None
        return ClientProxy(self._eng, cid)

    def teleport(self, pos):
        self._eng.world_entity_teleport(self.id, pos)

    def teleport_plane(self, stable_pid, pos):
        check_type(stable_pid, StablePlaneId)
        self._eng.world_entity_teleport_stable_plane(self.id, stable_pid, pos)

    def extra(self):
        return ExtraHashProxy(self._eng.world_entity_extra(self.id))

    inv = _inv
    create_inv = _create_inv


class InventoryProxy(ObjectProxy):
    ID_TYPE = InventoryId

    def attach(self, parent):
        self._eng.world_inventory_attach(self.id, parent)

    def count(self, item):
        item = DATA.item_id(item)
        return self._eng.world_inventory_count(self.id, item)

    def count_space(self, item):
        item = DATA.item_id(item)
        return self._eng.world_inventory_count_space(self.id, item)

    def bulk_add(self, item, count):
        item = DATA.item_id(item)
        return self._eng.world_inventory_bulk_add(self.id, item, count)

    def bulk_remove(self, item, count):
        item = DATA.item_id(item)
        return self._eng.world_inventory_bulk_remove(self.id, item, count)


class PlaneProxy(ObjectProxy):
    ID_TYPE = PlaneId

    def name(self):
        return self._eng.world_plane_name(self.id)

    def stable_id(self):
        return self._eng.world_plane_stable_id(self.id)

    def extra(self):
        return ExtraHashProxy(self._eng.world_plane_extra(self.id))

    def get_block(self, pos):
        block_id = self._eng.world_plane_get_block(self.id, pos)
        return BlockProxy.by_id(block_id)

    def find_structure_at_point(self, pos):
        opt_sid = self._eng.world_structure_find_at_point(self.id, pos)
        if opt_sid is not None:
            return StructureProxy(self._eng, opt_sid)
        else:
            return None

    def find_structure_at_point_layer(self, pos, layer):
        opt_sid = self._eng.world_structure_find_at_point_layer(self.id, pos, layer)
        if opt_sid is not None:
            return StructureProxy(self._eng, opt_sid)
        else:
            return None

    def create_structure(self, pos, template):
        template = DATA.template_id(template)
        sid = self._eng.world_structure_create(self.id, pos, template)
        return StructureProxy(self._eng, sid)

    def set_cave(self, pos):
        return self._eng.logic_set_cave(self.id, pos)

    def set_farmland(self, pos):
        self._eng.logic_set_interior(self.id, pos, 'farmland')

    def clear_farmland(self, pos):
        # TODO: strange design relative to other APIs, kind of a hack
        self._eng.logic_clear_interior(self.id, pos, 'farmland', 'grass/center/v0')


class StructureProxy(ObjectProxy):
    ID_TYPE = StructureId

    def destroy(self):
        self._eng.world_structure_destroy(self.id)

    def replace(self, template):
        template = DATA.template_id(template)
        self._eng.world_structure_replace(self.id, template)

    def stable_id(self):
        return self._eng.world_structure_stable_id(self.id)

    def pos(self):
        return self._eng.world_structure_pos(self.id)

    def plane(self):
        pid = self._eng.world_structure_plane_id(self.id)
        return PlaneProxy(self._eng, pid)

    def template(self):
        id = self._eng.world_structure_template_id(self.id)
        return TemplateProxy.by_id(id)

    def extra(self):
        return ExtraHashProxy(self._eng.world_structure_extra(self.id))

    inv = _inv
    create_inv = _create_inv


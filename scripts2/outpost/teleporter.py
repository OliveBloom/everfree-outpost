from outpost_server.core import use, util
from outpost_server.core.data import DATA

from outpost_server.outpost.lib import structure_items, tool, util as util2


ITEM = DATA.item('teleporter')
TEMPLATE = DATA.template('teleporter')

SETUP_DIALOG_ID = 1
DEST_DIALOG_ID = 2


class Networks(object):
    def __init__(self, eng):
        self.eng = eng

    def get_extra(self):
        return self.eng.world_extra().setdefault('teleport_networks', {})

    def has_endpoint(self, net, name):
        n = self.get_extra()
        return (net in n and name in n[net])

    def add_endpoint(self, net, name, pos):
        n = self.get_extra()
        if net not in n:
            n[net] = {}
        n[net][name] = pos

    def remove_endpoint(self, net, name):
        n = self.get_extra()
        del n[net][name]
        if len(n[net]) == 0:
            del n[net]

    def get_endpoint(self, net, name):
        n = self.get_extra()
        return n[net][name]

    def list_endpoints(self, net):
        n = self.get_extra()
        return sorted(n[net].keys())


@use.structure(TEMPLATE)
@util.with_args
def use_structure(e, s, args):
    n = Networks(e.engine)
    net = s.extra()['network']
    name = args['dest']

    if not n.has_endpoint(net, name):
        e.controller().send_message('The teleporter at %r has been disconnected.' % name)
        return

    pos = n.get_endpoint(net, name)
    e.teleport(pos)

@use_structure.get_args
def get_structure_args(e, s, args):
    n = Networks(e.engine)
    net = s.extra()['network']
    dests = n.list_endpoints(net)

    e.controller().get_interact_args(DEST_DIALOG_ID, {'dests': dests})


@use.item(ITEM)
@util.with_args
def use_item(e, args):
    util2.forest_check(e)

    n = Networks(e.engine)
    net = args['network']
    name = args['name']

    if n.has_endpoint(net, name):
        e.controller().send_message(
                'A teleporter named %r already exists on network %r.' % (name, net))
        return

    s = structure_items.place(e, ITEM, TEMPLATE)
    s.extra()['network'] = net
    s.extra()['name'] = name

    n.add_endpoint(net, name, e.pos())

@use_item.get_args
def get_item_args(e, args):
    util2.forest_check(e)

    e.controller().get_use_item_args(ITEM, SETUP_DIALOG_ID, {})


@tool.pickaxe(TEMPLATE)
def pickaxe_structure(e, s, args):
    n = Networks(e.engine)
    net = s.extra()['network']
    name = s.extra()['name']

    structure_items.take(e, s, ITEM)
    n.remove_endpoint(net, name)

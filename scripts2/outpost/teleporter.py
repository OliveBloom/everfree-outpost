from outpost_server.core import use, util
from outpost_server.core.data import DATA

from outpost_server.outpost.lib import structure_items, tool


ITEM = DATA.item('teleporter')
TEMPLATE = DATA.template('teleporter')

SETUP_DIALOG_ID = 1
DEST_DIALOG_ID = 2


def get_networks(eng):
    return eng.world_extra().setdefault('teleport_networks', {})

def has_endpoint(eng, net, name):
    n = get_networks(eng)
    return (net in n and name in n[net])

def add_endpoint(eng, net, name, pos):
    n = get_networks(eng)
    if net not in n:
        n[net] = {}
    n[net][name] = pos

def remove_endpoint(eng, net, name):
    n = get_networks(eng)
    del n[net][name]
    if len(n[net]) == 0:
        del n[net]

def get_endpoint(eng, net, name):
    n = get_networks(eng)
    return n[net][name]

def list_endpoints(eng, net):
    n = get_networks(eng)
    return sorted(n[net].keys())


@use.structure(TEMPLATE)
@util.with_args
def use_structure(e, s, args):
    eng = e.engine
    net = s.extra()['network']
    name = args['dest']

    if not has_endpoint(eng, net, name):
        e.controller().send_message('The teleporter at %r has been disconnected.' % name)
        return

    pos = get_endpoint(eng, net, name)
    e.teleport(pos)

@use_structure.get_args
def get_structure_args(e, s, args):
    eng = e.engine
    net = s.extra()['network']
    dests = list_endpoints(eng, net)

    e.controller().get_interact_args(DEST_DIALOG_ID, {'dests': dests})


@use.item(ITEM)
@util.with_args
def use_item(e, args):
    eng = e.engine
    net = args['network']
    name = args['name']

    if has_endpoint(eng, net, name):
        e.controller().send_message(
                'A teleporter named %r already exists on network %r.' % (name, net))
        return

    s = structure_items.place(e, ITEM, TEMPLATE)
    s.extra()['network'] = net
    s.extra()['name'] = name

    add_endpoint(eng, net, name, e.pos())

@use_item.get_args
def get_item_args(e, args):
    e.controller().get_use_item_args(ITEM, SETUP_DIALOG_ID, {})


@tool.pickaxe(TEMPLATE)
def pickaxe_structure(e, s, args):
    eng = e.engine
    net = s.extra()['network']
    name = s.extra()['name']

    structure_items.take(e, s, ITEM)
    remove_endpoint(eng, net, name)

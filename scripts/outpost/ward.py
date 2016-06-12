from outpost_server.core import chat, use, util
from outpost_server.core.data import DATA

from outpost_server.outpost.lib import structure_items, tool, util as util2, ward


ITEM = DATA.item('ward')
TEMPLATE = DATA.template('ward')


@use.item(ITEM)
def use_item(e, args):
    util2.forest_check(e)

    # TODO: hacky check for guest account status
    if ':' in e.controller().name():
        e.controller().send_message(
            'You must register an account before placing wards.')
        return

    pos = util.hit_tile(e)
    ok, msg = ward.can_add(e, pos)
    if not ok:
        e.controller().send_message(msg)
        return

    s = structure_items.place(e, ITEM, TEMPLATE)
    ward.add(e, pos, s)

@use.structure(TEMPLATE)
def use_structure(e, s, args):
    ok, msg = ward.can_remove(e, s)
    if not ok:
        e.controller().send_message(msg)
        return

    ward.remove(s)
    structure_items.take(e, s, ITEM, ignore_ward=True)


@chat.command('''
    /permit <name>: Give <name> permission to bypass your ward
    /revoke <name>: Revoke <name>'s permission to bypass your ward
''')
def permit(client, args):
    ward.permit(client.pawn(), args)
    client.send_message('Granted permission to %r' % args)

@chat.command(permit.doc)
def revoke(client, args):
    if ward.revoke(client.pawn(), args):
        client.send_message('Revoked permissions from %r' % args)
    else:
        client.send_message('No permissions to revoke from %r' % args)

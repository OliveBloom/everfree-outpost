from outpost_server.core import use
from outpost_server.core.data import DATA

from outpost_server.outpost.lib import structure_items, tool

SIGN_TEMPLATE = DATA.template('sign')
SIGN_ITEM = DATA.item('sign')
SIGN_TEXT_DIALOG_ID = 0

@use.structure(SIGN_TEMPLATE)
def sign_structure(e, s, args):
    e.controller().send_message('Sign: ' + s.extra()['message'])

@use.item(SIGN_ITEM)
def sign_item(e, args):
    if args is None:
        e.controller().get_use_item_args(SIGN_ITEM, SIGN_TEXT_DIALOG_ID, {})
    else:
        s = structure_items.place(e, SIGN_ITEM, SIGN_TEMPLATE)
        s.extra()['message'] = args['msg']

@tool.axe(SIGN_TEMPLATE)
def sign_axe(e, s, args):
    structure_items.take(e, s, SIGN_ITEM)

from outpost_server.core import use
from outpost_server.core.data import DATA

def register(item_name, what, where):
    item = DATA.item('blueprint/' + item_name)

    @use.item(item)
    def use_blueprint(e, args):
        if e.inv('ability').count(item) > 0:
            e.controller().send_message("You've already learned to craft %s." % what)
            return
        e.inv().bulk_remove(item, 1)
        e.inv('ability').bulk_add(item, 1)
        e.controller().send_message('You learn to craft %s.' % what)

    @use.ability(item)
    def use_ability(e, args):
        e.controller().send_message(
                'This is a passive ability.  Craft %s at %s.' % (what, where))

register('colored_torches', 'colored torches', 'an anvil')


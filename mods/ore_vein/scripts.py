from outpost_server.outpost.lib import tool, ward

@tool.pickaxe('ore_vein/copper')
def pickaxe_copper(e, s, args):
    ward.check(e, s.pos())
    if e.inv().count_space('ore/copper') == 0:
        return
    s.destroy()
    e.inv().bulk_add('ore/copper', 1)

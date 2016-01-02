def client_interact(eng, cid, args):
    print('interact', cid, args)
    eng.script_cb_interact(cid, args)

def client_use_item(eng, cid, item, args):
    print('use_item', cid, item, args)
    eng.script_cb_use_item(cid, item, args)

def client_use_ability(eng, cid, ability, args):
    print('use_ability', cid, ability, args)
    eng.script_cb_use_ability(cid, ability, args)

def init(hooks):
    hooks.client_interact(client_interact)
    hooks.client_use_item(client_use_item)
    hooks.client_use_ability(client_use_ability)

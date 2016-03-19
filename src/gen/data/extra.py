class ExtraDef(object):
    def __init__(self, name, func):
        self.name = name
        self.func = func

        self.value = None

    def resolve(self, id_maps):
        self.value = self.func(id_maps)

def resolve_all(extras, defs):
    for e in extras:
        e.resolve(defs)


# JSON output

def build_client_json(extras):
    return dict((e.name, e.value) for e in extras)

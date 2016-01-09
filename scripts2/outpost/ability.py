from outpost_server.core import use
from outpost_server.outpost.lib import appearance

@use.ability('ability/light')
def use_light(e, args):
    lit = appearance.get_light(e)
    appearance.set_light(e, not lit)

import functools

def energy_cost(amount):
    def decorate(f):
        @functools.wraps(f)
        def g(e, *args):
            if e.energy().take(amount):
                f(e, *args)
        return g
    return decorate

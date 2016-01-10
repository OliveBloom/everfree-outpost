def hit_pos(e):
    return e.pos() + 16 + e.facing() * 32

def hit_tile(e):
    return hit_pos(e).px_to_tile()

def hit_structure(e):
    return e.plane().find_structure_at_point(hit_tile(e))


class WithArgsWrapper(object):
    """Wrapper to simplify requesting additional arguments for a player action.
    When the wrapper is called, it checks whether the last argument is None,
    and calls either the `with_args` or `without_args` variant, passing through
    all arguments.

    Usage:
        @use.structure(TEMPLATE)
        @util.with_args     # NB: must be innermost decorator
        def use_structure(e, s, args):
            # This is the "with args" variant
            # do stuff with args[...]

        @use_structure.get_args
        def get_structure_args(e, s, args):
            # This is the "without args" variant
            e.controller().get_interact_args(...)
    """
    def __init__(self, f):
        self.with_args = f
        self.without_args = None

    def __call__(self, *args, **kwargs):
        if 'args' in kwargs:
            a = kwargs['args']
        else:
            a = args[-1]

        if a is None:
            return self.without_args(*args, **kwargs)
        else:
            return self.with_args(*args, **kwargs)

    def get_args(self, f):
        self.without_args = f
        return f

with_args = WithArgsWrapper

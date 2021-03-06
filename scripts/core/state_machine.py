from outpost_server.core import timer
from outpost_server.core.data import DATA
from outpost_server.core.engine import StructureProxy

KEY = 'sm'

class StateMachine:
    def __init__(self, obj):
        object.__setattr__(self, 'obj', obj)
        if KEY not in obj.extra():
            obj.extra()[KEY] = {}
            self.init()

    def __getattr__(self, k):
        try:
            return self.obj.extra()[KEY][k]
        except KeyError as e:
            raise AttributeError(str(e))

    def __setattr__(self, k, v):
        if k in self.__dict__:
            object.__setattr__(self, k, v)
        else:
            self.obj.extra()[KEY][k] = v

    def __delattr__(self, k):
        del self.obj.extra()[KEY][k]

    def _raw_get(self, k, default=None):
        try:
            return object.__getattribute__(self, k)
        except AttributeError:
            return default

    def init(self):
        self.state = type(self).START_STATE

    def process(self, msg):
        self.transition(msg)

    def transition(self, new_state):
        old_state = self.state

        f_leave = self._raw_get('leave_' + old_state)
        if f_leave is not None:
            f_leave()

        self.state = new_state

        f_enter = self._raw_get('enter_' + new_state)
        if f_enter is not None:
            f_enter()

    def schedule_at(self, when, msg):
        self.cancel()

        id = self.obj.id
        cookie = timer.schedule(self.obj.engine, when,
                lambda eng: callback(eng, id, when))
        self.timer = {
                'when': when,
                'cookie': cookie,
                'msg': msg,
                }

        # TODO: hack
        if isinstance(self.obj, StructureProxy):
            self.obj._eng.world_structure_set_has_import_hook(self.obj.id, True)

    def schedule(self, delay, msg):
        self.schedule_at(self.obj.engine.now() + delay, msg)

    def cancel(self):
        if hasattr(self, 'timer'):
            timer.cancel(self.obj.engine, self.timer['cookie'])
            del self.timer

        # TODO: hack
        if isinstance(self.obj, StructureProxy):
            self.obj._eng.world_structure_set_has_import_hook(self.obj.id, False)

_TEMPLATE_SM = {}

def structure(template):
    template = DATA.template(template)
    def f(cls):
        assert template not in _TEMPLATE_SM, \
                'duplicate registration for %s' % template
        _TEMPLATE_SM[template] = cls
        return cls
    return f

def get(obj):
    cls = state_machine_class(obj)
    return cls(obj)

def state_machine_class(obj):
    if isinstance(obj, StructureProxy):
        return _TEMPLATE_SM[obj.template()]

def get_state_machine_class(obj):
    if isinstance(obj, StructureProxy):
        return _TEMPLATE_SM.get(obj.template())

def callback(eng, id, when):
    obj = eng.get_object(id)
    cls = get_state_machine_class(obj)
    if cls is None:
        return

    timer = obj.extra().get(KEY, {}).get('timer')
    if timer is None:
        return
    if when != timer['when']:
        raise ValueError('mismatched when (old timer fired after being cancelled)')
    msg = timer['msg']
    del obj.extra()[KEY]['timer']

    # TODO: hack
    if isinstance(obj, StructureProxy):
        obj._eng.world_structure_set_has_import_hook(obj.id, False)

    cls(obj).process(msg)


def structure_import_hook(eng, sid):
    s = StructureProxy(eng, sid)
    t = s.extra().get('sm', {}).get('timer')
    if t is not None:
        when = t['when']
        cookie = timer.schedule(s.engine, when, 
                lambda eng: callback(eng, sid, when))
        s.extra()['sm']['timer']['cookie'] = cookie

def init(hooks):
    hooks.structure_import_hook(structure_import_hook)

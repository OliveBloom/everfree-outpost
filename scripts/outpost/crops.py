import random

from outpost_server.core import state_machine, use, util
from outpost_server.core.data import DATA

from outpost_server.outpost.chest import register_crate_variant
from outpost_server.outpost.lib import appearance, structure_items, ward

def register(basename):
    t = [DATA.template('%s/%d' % (basename, i)) for i in range(4)]
    item = DATA.item(basename)

    register_crate_variant(basename)

    @state_machine.structure(t[0])
    @state_machine.structure(t[1])
    @state_machine.structure(t[2])
    @state_machine.structure(t[3])
    class CropMachine(state_machine.StateMachine):
        def init(self):
            self.base = None
            self.step = None

        def start(self, step):
            self.step = step
            self.base = self.obj.engine.now()
            self.transition(0)

        def transition(self, state):
            self.obj.replace(t[state])

            if state < 3:
                next_state = state + 1
                when = self.base + next_state * self.step
                self.schedule_at(when, next_state)

    @use.item(item)
    def plant_crop(e, args):
        if not e.plane().get_block(util.hit_tile(e)).name.startswith('farmland/'):
            if e.inv().count(item) == 0:
                raise RuntimeError('missing item in inventory')
            gave = e.energy().give(10)
            if gave > 0:
                e.inv().bulk_remove(item, 1)
            return

        s = structure_items.place(e, item, t[0])

        step_base = 6 if appearance.is_tribe(e, 'E') else 8
        step_s = step_base * 60 + random.randrange(-30, 30)

        state_machine.get(s).start(step_s * 1000)

    @use.structure(t[0])
    @use.structure(t[1])
    @use.structure(t[2])
    def destroy_crop(e, s, args):
        ward.check(e, s.pos())
        s.destroy()

    @use.structure(t[3])
    def harvest_crop(e, s, args):
        ward.check(e, s.pos())
        if e.inv().count_space(item) == 0:
            raise RuntimeError('no space for item in inventory')
        s.destroy()
        e.inv().bulk_add(item, random.randrange(1, 4))

register('tomato')
register('potato')
register('carrot')
register('artichoke')
register('pepper')
register('cucumber')
register('corn')

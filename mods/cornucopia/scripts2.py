import random
from outpost_server.core import use

FOODS = (
        'tomato',
        'potato',
        'carrot',
        'artichoke',
        'pepper',
        'cucumber',
        'corn',
    )

@use.structure('cornucopia')
def use_cornucopia(e, s, args):
    if 'used_cornucopia' in e.extra():
        return
    food = random.choice(FOODS)
    e.inv().bulk_add(food, 5)
    e.extra()['used_cornucopia'] = True

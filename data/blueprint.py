from outpost_data.core.consts import *
from outpost_data.core.builder2 import *
from outpost_data.core.image2 import load

def do_blueprint(icon, name, disp_name):
    ITEM.new('blueprint/' + name) \
            .display_name('Blueprint: ' + disp_name) \
            .icon(icon)

def init():
    icon = load('icons/blueprint.png')

    do_blueprint(icon, 'colored_torches', 'Colored Torches')

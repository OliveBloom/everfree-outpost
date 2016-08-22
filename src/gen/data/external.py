import yaml

from outpost_data.core import files


ITEM_DESCS = {}

def get_item_desc(name):
    return ITEM_DESCS.get(name)

def load():
    for path in files.find_all('item_descs.yaml'):
        with open(path) as f:
            dct = yaml.load(f)
            ITEM_DESCS.update(dct)

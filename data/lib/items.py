from PIL import Image

from ...core.builder import *
from ...core.consts import *
from ...core.structure import StructureDef
from ...core.util import extract

from outpost_data.core.builder2 import ITEM
from outpost_data.core.builder2.structure import StructureBuilder


def mk_structure_item(s, name, ui_name, base=None):
    if isinstance(s, Structures):
        s = s.unwrap()

    if isinstance(s, StructureBuilder):
        s = s.unwrap()

    if isinstance(s, StructureDef):
        if base is None:
            orig = s.get_image()
            w, h = orig.size
            side = max(w, h)
            img = Image.new('RGBA', (side, side))
            img.paste(orig, ((side - w) // 2, (side - h) // 2))
            img = img.resize((TILE_SIZE, TILE_SIZE), resample=Image.BILINEAR)
        else:
            img = extract(s.get_image(), base)

        return mk_item(name, ui_name, img)
    else:
        b = item_builder()
        b._builder = ITEM.from_structure(s, name=name).display_name(ui_name)
        return b


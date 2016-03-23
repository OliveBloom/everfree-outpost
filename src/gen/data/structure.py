from collections import namedtuple

from PIL import Image, ImageChops, ImageDraw

from outpost_data.core import geom, image2, image_cache, util
from outpost_data.core.consts import *


class Shape(object):
    def __init__(self, x, y, z, arr):
        self.size = (x, y, z)
        self.shape_array = arr

def empty(x, y, z):
    arr = ['empty'] * (x * y * z)
    return Shape(x, y, z, arr)

def floor(x, y, z):
    arr = ['floor'] * (x * y) + ['empty'] * (x * y * (z - 1))
    return Shape(x, y, z, arr)

def solid(x, y, z):
    arr = ['solid'] * (x * y * z)
    return Shape(x, y, z, arr)


class Model(object):
    def __init__(self, verts):
        self.verts = verts
        self.length = len(verts)

    def slice_xv(self, offset, size, base_z):
        min_x, min_v = offset
        min_v -= base_z * TILE_SIZE
        size_x, size_v = size
        max_x = min_x + size_x
        max_v = min_v + size_v
        verts = [p
                for tri in geom.slice_tris_box_xv(self.verts, min_x, max_x, min_v, max_v)
                for p in tri]

        return Model(verts)

    def to_mesh(self):
        m = geom.Mesh()
        for i in range(0, len(self.verts), 3):
            m.add_tri(*self.verts[i : i + 3])
        return m

Model2 = namedtuple('Model2', ('mesh', 'bounds'))

class StructurePart(object):
    def __init__(self, model2, img):
        mesh = model2.mesh.copy()

        # Compute bounding box of the (projection of the) mesh
        v2_min, v2_max = mesh.get_bounds_2d(util.project)
        v2_size = (v2_max[0] - v2_min[0], v2_max[1] - v2_min[1])

        # The user provided the bounding box of the region covered by the
        # image.  (Usually this is just (0,0,0) .. (struct_size).)  Using that,
        # extract the image for the part actually covered by the mesh.
        b_min, b_max = model2.bounds
        b2_min = (b_min[0], b_min[1] - b_max[2])
        b2_max = (b_max[0], b_max[1] - b_min[2])
        b2_size = (b2_max[0] - b2_min[0], b2_max[1] - b2_min[1])

        # Crop image to the mesh's bounding box

        # Special hack
        if img.px_size == (TILE_SIZE, TILE_SIZE) and \
                b2_size == (TILE_SIZE, 2 * TILE_SIZE):
            pass
        else:
            assert img.px_size == b2_size, \
                    'image has wrong size for bounds: %s != %s' % (img.px_size, b2_size)

            # Extract the region (v2_min - b2_min, v2_max - b2_min)
            r2_min = (v2_min[0] - b2_min[0], v2_min[1] - b2_min[1])
            img = img.extract(r2_min, size=v2_size, unit=1)

        # Crop mesh & image to image's bounding box.  We do this after the
        # previous step because cropping the image may remove some non-blank
        # content, letting us crop the image more aggressively here.
        img, i2_min = img.autocrop()
        v2_min = geom.add(v2_min, i2_min)
        v2_max = geom.add(v2_min, img.px_size)
        geom.clip_xv(mesh, *(v2_min + v2_max))

        self.mesh = mesh
        self.img = img
        self.base = v2_min

        # Set by `build_sheets`
        self.sheet_idx = None
        self.offset = None

        # Set by `collect_verts'
        self.vert_idx = None
        self.vert_count = None

    def get_sheet_image(self):
        if isinstance(self.img, image2.Anim):
            return self.img.flatten()
        else:
            return self.img

    def get_flags(self):
        anim = False
        shadow = False

        if isinstance(self.img, image2.Anim):
            anim = True
            # Animated parts can't have shadows.
        elif isinstance(self.img, image2.Image):
            img = self.img.raw().raw()
            if img.mode == 'RGBA':
                hist = img.histogram()
                a_hist = hist[256 * 3 : 256 * 4]
                # Is there any pixel whose alpha value is not 0 or 255?
                shadow = any(count > 0 for count in a_hist[1:-1])
            else:
                shadow = False

        return ((int(shadow) << 0) |
                (int(anim) << 1))


class StructureDef2(object):
    def __init__(self, name, shape, layer):
        self.name = name
        self.size = shape.size
        self.shape = shape.shape_array
        self.layer = layer
        self.parts = []

        self.light_pos = None
        self.light_color = None
        self.light_radius = None

        # Filled in by `assign_ids`
        self.id = None

        # Filled in by `collect_parts`
        self.part_idx = None
        
        # Filled in by `collect_shapes`
        self.shape_idx = None

    '''
    def get_display_px(self):
        """Get the size of the structure on the screen."""
        return self.image.size

    def get_sheet_px(self):
        """Get the size of the structure in the sprite sheet.  Currently this
        is the same as `get_display_px` because animated structures place only
        the static parts in the main structure sheet."""
        return self.image.size

    def get_display_size(self):
        w, h = self.get_display_px()
        return (w // TILE_SIZE, h // TILE_SIZE)

    def get_sheet_size(self):
        w, h = self.get_sheet_px()
        return (w // TILE_SIZE, h // TILE_SIZE)

    def get_anim_sheet_px(self):
        return self.anim.anim_sheet.size

    def get_anim_sheet_size(self):
        w, h = self.get_anim_sheet_px()
        return ((w + TILE_SIZE - 1) // TILE_SIZE,
                (h + TILE_SIZE - 1) // TILE_SIZE)
    '''

    def add_part(self, model2, img):
        self.parts.append(StructurePart(model2, img))

    def set_light(self, pos, color, radius):
        self.light_pos = pos
        self.light_color = color
        self.light_radius = radius

    def get_flags(self):
        light = self.light_radius is not None
        flags = (int(light) << 2)
        for p in self.parts:
            flags = flags | p.get_flags()
        return flags


# Sprite sheets

def build_sheets(structures):
    '''Build sprite sheet(s) containing all the images for the provided
    structures.  This also updates each part's `sheet_idx` and `offset` field
    with its position in the generated sheet(s).
    '''
    parts = [p for s in structures for p in s.parts]
    all_imgs = [p.get_sheet_image() for p in parts]
    imgs, idx_map = util.dedupe(all_imgs, [i.hash() for i in all_imgs])
    boxes = [tuple((x + TILE_SIZE - 1) // TILE_SIZE for x in i.size) for i in imgs]
    num_sheets, offsets = util.pack_boxes(SHEET_SIZE, boxes)

    sheets = [Image.new('RGBA', (SHEET_PX, SHEET_PX)) for _ in range(num_sheets)]

    for img, (j, offset) in zip(imgs, offsets):
        x, y = offset
        x *= TILE_SIZE
        y *= TILE_SIZE

        sheets[j].paste(img.raw().raw(), (x, y))

    for part, img in zip(parts, all_imgs):
        idx = idx_map[id(img)]
        j, offset = offsets[idx]

        part.sheet_idx = j
        part.offset = geom.mul(offset, TILE_SIZE)

    return sheets

def collect_parts(structures):
    all_parts = []

    for s in structures:
        s.part_idx = len(all_parts)
        all_parts.extend(s.parts)

    return all_parts

def collect_verts(parts):
    idx_map = {}
    all_verts = []

    for p in parts:
        k = tuple(v.pos for v in p.mesh.iter_tri_verts())
        if k not in idx_map:
            idx_map[k] = len(all_verts)
            all_verts.extend(k)
        p.vert_idx = idx_map[k]
        p.vert_count = len(k)

    return all_verts

def collect_shapes(structures):
    idx_map = {}
    all_shapes = []

    for s in structures:
        k = tuple(SHAPE_ID[x] for x in s.shape)
        if k not in idx_map:
            idx_map[k] = len(all_shapes)
            all_shapes.extend(k)
        s.shape_idx = idx_map[k]

    return all_shapes


# JSON output

def build_client_json(structures):
    def convert(s):
        dct = {
                'size': s.size,
                'shape': [SHAPE_ID[x] for x in s.shape],
                'shape_idx': s.shape_idx,
                'part_idx': s.part_idx,
                'part_count': len(s.parts),
                'vert_count': sum(p.vert_count for p in s.parts),
                'layer': s.layer,
                }

        flags = s.get_flags()
        if flags != 0:
            dct.update(
                flags=flags)

        if s.light_color is not None:
            dct.update(
                light_pos=s.light_pos,
                light_color=s.light_color,
                light_radius=s.light_radius)

        return dct

    return list(convert(s) for s in structures)

def build_parts_json(parts):
    def convert(p):
        dct = {
                'sheet': p.sheet_idx,
                # Give the offset of the pixel corresponding to (0,0,0)
                'offset': geom.sub(p.offset, p.base),
                'vert_idx': p.vert_idx,
                'vert_count': p.vert_count,
                }

        flags = p.get_flags()
        if flags != 0:
            dct.update(
                flags=flags)

        if isinstance(p.img, image2.Anim):
            dct.update(
                anim_length=p.img.length,
                anim_rate=p.img.rate,
                anim_oneshot=p.img.oneshot,
                anim_size=p.img.px_size)

        return dct

    return list(convert(p) for p in parts)

def build_verts_json(verts):
    return list(x for v in verts for x in v)

def build_shapes_json(shapes):
    return list(shapes)

def build_server_json(structures):
    def convert(s):
        return {
                'name': s.name,
                'size': s.size,
                'shape': [SHAPE_ID[x] for x in s.shape],
                'layer': s.layer,
                }

    return list(convert(s) for s in structures)

import base64
import json
import os
import sys
from PIL import Image

from equip_sprites_render import Renderer

FRAMES = []
def add_anim(name, x, y, length):
    for i in range(length):
        FRAMES.append(('%s$%d' % (name, i), x + i, y))

for (d, d_off) in [(0, 2), (1, 4), (3, 0)]:
    add_anim('stand-%d' % d, d_off + 0, 0, 1)
    add_anim('sit-%d' % d, d_off + 5, 0, 1)
    add_anim('walk-%d' % d, d_off * 6, 1, 6)
    add_anim('run-%d' % d, d_off * 6, 3, 6)

# sleep-0 only has three frames in the sheets, but appears in the data as six
# frames.  It repeats 0, 0, 1, 2, 2, 1.
FRAMES.extend([
    ('sleep-0$0', 10, 0),
    ('sleep-0$2', 11, 0),
    ('sleep-0$4', 12, 0),
    ])


STYLES = [
        'solid/red',
        'solid/orange',
        'solid/yellow',
        'solid/green',
        'solid/blue',
        'solid/purple',
        'solid/black',
        'solid/white',
        ]

def main(json_path, base_img_path, layer_name, out_dir):
    sprite_name = layer_name[:layer_name.index('//')]

    os.makedirs(out_dir, exist_ok=True)


    r = Renderer()

    img = Image.open(base_img_path)
    r.set_base(img.size[0], img.size[1], img.tobytes())

    with open(json_path) as f:
        j = json.load(f)

    out = Image.new('RGBA', img.size)

    for frame, x, y in FRAMES:
        frame_base = img.crop((x * 96, y * 96, (x + 1) * 96, (y + 1) * 96))
        r.set_base(96, 96, frame_base.tobytes())

        for i in (3, 2, 1, 0):
            key = '%s$%s//%s$%d' % (layer_name, sprite_name, frame, i)
            if key not in j:
                continue
            mask = base64.b64decode(j[key])
            r.render_part(mask)

        frame_out = Image.frombytes('RGBA', frame_base.size, r.get_image())
        out.paste(frame_out, (x * 96, y * 96))

    out.save(os.path.join(out_dir, 'solid_red.png'))

if __name__ == '__main__':
    json_path, base_img_path, layer_name, out_dir = sys.argv[1:]
    main(json_path, base_img_path, layer_name, out_dir)

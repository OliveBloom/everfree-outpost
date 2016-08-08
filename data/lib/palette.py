from outpost_data.core.image2 import load


def load_palettes(img):
    def load_palettes_inner(raw):
        w, h = raw.size
        pals = []
        for x in range(0, w, 2):
            pals.append([raw.getpixel((x, y)) for y in range(0, h, 2)])
        return pals
    return img.raw().compute(load_palettes_inner)


METALS = (
        '_base',
        'stone',
        'copper',
        'bronze',
        'iron',
        't1_earth',
        't2_earth',
        't1_pegasus',
        't2_pegasus',
        't1_unicorn',
        't2_unicorn',
        'silver',
        )

METAL_PALETTES = dict(zip(METALS,
    load_palettes(load('misc/metal-palettes.png'))))


def collect_mask_colors(img):
    def collect_mask_colors_inner(raw):
        quant = raw.quantize()
        pal = quant.getpalette()
        levels = []
        for i in range(0, len(pal), 3):
            if pal[i] == pal[i + 2] and pal[i + 1] == 0 and pal[i] != 0:
                levels.append(pal[i])
        levels.sort(reverse=True)
        return [(x, 0, x, 255) for x in levels]
    return img.raw().compute(collect_mask_colors_inner)

def recolor(img, palette, base_palette=None):
    if base_palette is None:
        base_palette = collect_mask_colors(img)
    palette = tuple(tuple(x[:3]) for x in palette)
    base_palette = tuple(tuple(x[:3]) for x in base_palette)

    dct = dict(zip(base_palette, palette))

    def recolor_inner(raw):
        quant = raw.quantize()
        if raw.mode == 'RGBA':
            alpha = raw.split()[3]

        pal = list(quant.getpalette())
        for i in range(0, len(pal), 3):
            k = tuple(pal[i : i + 3])
            if k in dct:
                pal[i : i + 3] = dct[k]

        quant.putpalette(pal)
        new_img = quant.convert(raw.mode)
        if raw.mode == 'RGBA':
            new_img.putalpha(alpha)
        return new_img

    return img.modify(recolor_inner,
            desc=(__name__, 'recolor', palette, base_palette))



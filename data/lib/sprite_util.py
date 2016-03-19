import PIL.Image
import PIL.ImageChops


def _set_depth(img, depth):
    old_alpha = img.split()[3]
    mask = PIL.Image.new('L', img.size, depth)
    img.putalpha(PIL.ImageChops.darker(mask, old_alpha))
    return img

def set_depth(img, depth):
    return img.modify(f=lambda raw: _set_depth(raw, depth),
            desc=('sprites2.set_depth', depth))

def depth_stack(img_depths):
    imgs = tuple(i for i, d in img_depths)
    depths = tuple(d for i, d in img_depths)
    assert len(imgs) > 0

    def f(*args):
        acc = PIL.Image.new('RGBA', args[0].size)
        for img, depth in zip(args, depths):
            # Extract original alpha channel
            mask = img.split()[3].copy()
            # Set alpha to depth, uniformly
            img.putalpha(PIL.Image.new('L', img.size, depth))
            # Overwrite `acc` with `img` (including alpha), filtered by `mask`
            acc.paste(img, (0, 0), mask)
        return acc

    return imgs[0].fold(imgs[1:], f=f, desc=(__name__ + '.depth_stack', depths))



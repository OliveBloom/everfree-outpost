from outpost_data.core.image2 import Image

def vec_div(v, c):
    x, y = v
    return ((x + c - 1) // c, (y + c - 1) // c)

def vec_mul(v, c):
    x, y = v
    return (x * c, y * c)


class Page(object):
    def __init__(self, size):
        self.w, self.h = size
        self.in_use = 0
        self.avail_area = self.w * self.h

    def place(self, box):
        w, h = box

        base_mask = (1 << w) - 1
        mask = 0
        for i in range(h):
            mask |= base_mask << (i * self.w)

        for y in range(0, self.h - h + 1):
            base_i = y * self.w
            for x in range(0, self.w - w + 1):
                i = base_i + x
                if (mask << i) & self.in_use == 0:
                    self.in_use |= mask << i
                    self.avail_area -= w * h
                    return (x, y)

        return None


class BoxPacker:
    def __init__(self, page_size, res=1):
        self.res = res
        self.page_size = vec_div(page_size, self.res)
        self.pages = [Page(self.page_size)]

    def _div(self, v):
        return vec_div(v, self.res)

    def _mul(self, v):
        return vec_mul(v, self.res)

    def place(self, boxes):
        """Pack a list of boxes (`(w, h)` pairs) into the generated pages."""
        boxes = [self._div(box) for box in boxes]

        # Sort by decreasing size
        def key(b):
            i, (w, h) = b
            return (w * h, h, w)
        boxes = sorted(enumerate(boxes), key=key, reverse=True)

        result = [None] * len(boxes)
        for i, box in boxes:
            w,h = box
            for j, p in reversed(list(enumerate(self.pages))):
                if p.avail_area < w * h:
                    continue
                pos = p.place(box)
                if pos is not None:
                    result[i] = (j, self._mul(pos))
                    break
            else:
                # The loop didn't `break`, so the box didn't fit on any existing page.
                self.pages.append(Page(self.page_size))
                pos = self.pages[-1].place(box)
                assert pos is not None, \
                        'box is too large to fit on a page (%s > %s)' % \
                        (self._mul(box), self._mul(self.page_size))
                result[i] = (len(pages) - 1, self._mul(pos))

        return result

    def num_pages(self):
        return len(self.pages)

class ImagePacker:
    def __init__(self, page_size, res=1):
        self.boxes = BoxPacker(page_size, res=res)
        self.images = []
        self.px_size = page_size

    def place(self, imgs):
        boxes = [i.px_size for i in imgs]
        offsets = self.boxes.place(boxes)
        self.images.extend(zip(imgs, offsets))
        return offsets

    def build_sheets(self):
        return [
                Image.sheet([(img, off) for img, (sheet, off) in self.images if sheet == i],
                    self.px_size)
                for i in range(self.boxes.num_pages())
                ]


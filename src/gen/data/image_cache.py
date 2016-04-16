import base64
import functools
import hashlib
import pickle
import os
import sys
from weakref import WeakValueDictionary

import PIL


IMAGE_CACHE = None
COMPUTE_CACHE = None

@functools.lru_cache(128)
def _cached_mtime(path):
    return os.path.getmtime(path)

class CachedImage(object):
    """An immutable wrapper around PIL.Image that allows for caching of
    intermediate images."""
    def __init__(self, size, desc, inputs):
        self.size = size
        self._desc = (type(self), desc, tuple(i._desc for i in inputs))
        self._raw = None

    def _realize(self):
        raise RuntimeError('CachedImage subclass must implement _realize()')

    def raw(self):
        img = self._raw
        if img is not None:
            return img

        img = IMAGE_CACHE.get(self._desc)
        if img is not None:
            assert img.size == self.size, 'cache contained an image of the wrong size'
            self._raw = img
            return img

        img = self._realize()
        assert img is not None, '_realize() must return a PIL.Image, not None'
        assert img.size == self.size, '_realize() returned an image of the wrong size'
        self._raw = img
        if type(img) is not PIL.Image.Image:
            # It's a lazy crop object, or something similar.  Force it.
            img = img.copy()
        IMAGE_CACHE.add(self._desc, img)
        return img

    def compute(self, f, desc=None):
        code_file = sys.modules[f.__module__].__file__
        code_time = _cached_mtime(code_file)
        if desc is None:
            desc = (f.__module__, f.__qualname__)
        k = ('compute', self._desc, desc, code_file, code_time)

        # Avoid .get because a compute result may legitimately be `None`
        if COMPUTE_CACHE.contains(k):
            result = COMPUTE_CACHE.get(k)
        else:
            result = f(self.raw())
            COMPUTE_CACHE.add(k, result)

        return result

    def desc(self):
        return self._desc

    @staticmethod
    def blank(size):
        return BlankImage(size)

    @staticmethod
    def open(filename):
        return FileImage(filename)

    @staticmethod
    def from_raw(img):
        return ConstImage(img)

    def modify(self, f, size=None, desc=None):
        if desc is None:
            desc = '%s.%s' % (f.__module__, f.__qualname__)
        return ModifiedImage(self, f, size or self.size, desc)

    def fold(self, imgs, f, size=None, desc=None):
        if desc is None:
            desc = '%s.%s' % (f.__module__, f.__qualname__)
        return FoldedImage(self, imgs, f, size or self.size, desc)

    def crop(self, bounds):
        return CroppedImage(self, bounds)

    def resize(self, size, resample=0):
        return ResizedImage(self, size, resample)

    def stack(self, imgs):
        return StackedImage((self,) + tuple(imgs))

    def pad(self, size, offset):
        return PaddedImage(self, size, offset)

    @staticmethod
    def sheet(img_offsets, size=None):
        if size is None:
            w, h = 0, 0
            for i, o in img_offsets:
                w = max(w, i.size[0] + o[0])
                h = max(h, i.size[1] + o[1])
            size = (w, h)

        return SheetImage(img_offsets, size)

    def get_bounds(self):
        # NB: we only consider the alpha channel when finding the bounds.  This
        # means pixels with zero alpha but non-zero color will be considered
        # empty.
        b = self.compute(lambda i: i.split()[3].getbbox())
        if b is None:
            return (0, 0, 0, 0)
        else:
            return b

class BlankImage(CachedImage):
    def __init__(self, size):
        super(BlankImage, self).__init__(size, size, ())

    def _realize(self):
        return PIL.Image.new('RGBA', self.size)

class ConstImage(CachedImage):
    def __init__(self, img):
        h = hashlib.sha1(bytes(x for p in img.getdata() for x in p)).hexdigest()
        super(ConstImage, self).__init__(img.size, (img.size, h), ())
        self._raw = img

    def _realize(self):
        assert False, 'ConstImage already sets self._raw, should be no need to call _realize()'

class FileImage(CachedImage):
    def __init__(self, filename):
        mtime = os.path.getmtime(filename)
        img = PIL.Image.open(filename)
        super(FileImage, self).__init__(img.size, (filename, mtime), ())
        self._raw = img

    def _realize(self):
        assert False, 'FileImage already sets self._raw, should be no need to call _realize()'

class ModifiedImage(CachedImage):
    def __init__(self, img, f, size, desc):
        code_file = sys.modules[f.__module__].__file__
        code_time = _cached_mtime(code_file)

        super(ModifiedImage, self).__init__(size, (desc, size, code_time), (img,))
        self.orig = img
        self.f = f

    def _realize(self):
        img = self.orig.raw().copy()
        return self.f(img) or img

class FoldedImage(CachedImage):
    def __init__(self, base_img, imgs, f, size, desc):
        code_file = sys.modules[f.__module__].__file__
        code_time = _cached_mtime(code_file)

        imgs = tuple(imgs)
        super(FoldedImage, self).__init__(size, (desc, size, code_time), (base_img,) + imgs)
        self.base_orig = base_img
        self.origs = imgs
        self.f = f

    def _realize(self):
        base = self.base_orig.raw().copy()
        imgs = [o.raw().copy() for o in self.origs]
        return self.f(base, *imgs) or base

class CroppedImage(CachedImage):
    def __init__(self, img, bounds):
        x0, y0, x1, y1 = bounds
        w = x1 - x0
        h = y1 - y0

        super(CroppedImage, self).__init__((w, h), bounds, (img,))

        self.orig = img
        self.bounds = bounds

    def _realize(self):
        return self.orig.raw().crop(self.bounds)

class ResizedImage(CachedImage):
    def __init__(self, img, size, resample=0):
        super(ResizedImage, self).__init__(size, (size, resample), (img,))

        self.orig = img
        # self.size already set
        self.resample = resample

    def _realize(self):
        return self.orig.raw().resize(self.size, self.resample)

class StackedImage(CachedImage):
    def __init__(self, imgs):
        assert all(i.size == imgs[0].size for i in imgs)
        super(StackedImage, self).__init__(imgs[0].size, (), imgs)
        self.imgs = imgs

    def _realize(self):
        img = self.imgs[0].raw().copy()
        for i in self.imgs[1:]:
            layer_img = i.raw()
            img.paste(layer_img, (0, 0), layer_img)
        return img

class PaddedImage(CachedImage):
    def __init__(self, img, size, offset):
        super(PaddedImage, self).__init__(size, (size, offset), (img,))
        self.orig = img
        # self.size already set
        self.offset = offset

    def _realize(self):
        orig_img = self.orig.raw()
        img = PIL.Image.new(orig_img.mode, self.size)
        img.paste(orig_img, self.offset)
        return img

class SheetImage(CachedImage):
    def __init__(self, img_offsets, size):
        imgs = tuple(i for i,o in img_offsets)
        offsets = tuple(o for i,o in img_offsets)
        super(SheetImage, self).__init__(size, (offsets,), imgs)

        self.imgs = imgs
        self.offsets = offsets

    def _realize(self):
        acc = PIL.Image.new('RGBA', self.size)
        for i, o in zip(self.imgs, self.offsets):
            acc.paste(i.raw(), o)
        return acc


WORKAROUND_0X0 = 'workaround-0x0-bug'

def _safe_dump(value, f):
    if isinstance(value, PIL.Image.Image) and value.size == (0, 0):
        # Pickling a 0x0 image seems to cause a crash ("tile cannot extend
        # outside image").  Store this dummy value instead.
        pickle.dump(WORKAROUND_0X0, f)
    else:
        pickle.dump(value, f)

def _safe_load(f):
    value = pickle.load(f)
    if value == WORKAROUND_0X0:
        return PIL.Image.new('RGBA', (0, 0))
    else:
        return value

CACHE_PAGE = 4096

class LargeCache:
    '''File-backed cache for large objects (particularly images).  Uses a
    WeakValueDictionary for in-memory storage, and a page-based format for
    on-disk.'''
    def __init__(self, data_file, index_file):
        # Dict storing currently loaded values
        self.cache = WeakValueDictionary()

        # Data file, and index mapping key to data offset
        self.data_file = data_file
        self.data_total = os.fstat(data_file.fileno()).st_size

        self.index_file = index_file
        self.index = {}

        self.used = set()

        # Read index data into dict
        self.index_file.seek(0)
        for line in self.index_file.readlines():
            parts = line.strip().split()
            if len(parts) != 2:
                continue
            offset_str, key_str = parts
            offset = int(offset_str)
            key = pickle.loads(base64.decodebytes(key_str.encode('ascii')))
            self.index[key] = offset

        # Seek both to EOF
        self.data_file.seek(0, os.SEEK_END)
        self.index_file.seek(0, os.SEEK_END)

    def contains(self, key):
        return key in self.cache or key in self.index

    def get(self, key):
        # Record the key use here, regardless of the outcome.  This assumes the
        # caller runs `add` only on keys for which it first ran `get`.
        self.used.add(key)

        # Try to fetch from in-memory cache
        value = self.cache.get(key)
        if value is not None:
            return value

        # Try to load from file
        if key in self.index:
            offset = self.index[key]
            self.data_file.seek(offset)
            value = _safe_load(self.data_file)
            self.cache[key] = value
            return value

        # No cached copy of this image
        return None

    def add(self, key, value):
        # Add to in-memory cache
        self.cache[key] = value

        # Write pickled value to next available page
        page = CACHE_PAGE
        offset = (self.data_total + page - 1) & ~(page - 1)
        self.data_file.seek(offset)
        _safe_dump(value, self.data_file)
        self.data_total = self.data_file.tell()

        # Write index line
        self.index[key] = offset
        key_str = base64.encodebytes(pickle.dumps(key)).decode('ascii')
        self.index_file.write('%d %s\n' % (offset, key_str.replace('\n', '')))

    def size(self):
        return len(self.index)

    def save(self):
        pass

class SmallCache:
    '''File-backed cache for small objects.  Uses a standard dict for in-memory
    storage and a single pickle file on disk.'''
    def __init__(self, data_file):
        self.data_file = data_file

        self.data_file.seek(0)
        try:
            self.dct = pickle.load(self.data_file)
        except:
            self.dct = {}

    def contains(self, key):
        return key in self.dct

    def get(self, key):
        return self.dct.get(key)

    def add(self, key, value):
        self.dct[key] = value

    def size(self):
        return len(self.dct)

    def save(self):
        self.data_file.seek(0)
        self.data_file.truncate(0)
        pickle.dump(self.dct, self.data_file)


def load_cache(cache_dir):
    global IMAGE_CACHE, COMPUTE_CACHE

    def open2(name, binary=False):
        b = 'b' if binary else ''
        try:
            # Open without truncating
            return open(os.path.join(cache_dir, name), 'r+' + b)
        except OSError:
            # Doesn't exist, so create it
            return open(os.path.join(cache_dir, name), 'w+' + b)

    IMAGE_CACHE = LargeCache(
            open2('image_cache.dat', binary=True),
            open2('image_cache.idx'))
    COMPUTE_CACHE = SmallCache(
            open2('compute_cache.dat', binary=True))

def new_cache(cache_dir):
    def rm(name):
        path = os.path.join(cache_dir, name)
        if os.path.exists(path):
            os.unlink(path)

    rm('image_cache.dat')
    rm('image_cache.idx')
    rm('compute_cache.dat')
    load_cache(cache_dir)

def save_cache():
    IMAGE_CACHE.save()
    COMPUTE_CACHE.save()

def _old_dump_cache(f):
    global NEW_IMAGE_CACHE, NEW_COMPUTE_CACHE
    for k, v in NEW_IMAGE_CACHE.items():
        new_v = v
        if type(new_v) is not PIL.Image.Image:
            # It may be an _ImageCrop or similar.
            new_v = v.copy()
            new_v.load()

        if new_v.size == (0, 0):
            # Pickling a 0x0 image seems to cause a crash ("tile cannot extend
            # outside image").  Store this dummy value instead.
            new_v = WORKAROUND_0X0

        if new_v is not v:
            NEW_IMAGE_CACHE[k] = new_v

    blob = (NEW_IMAGE_CACHE, NEW_COMPUTE_CACHE)
    pickle.dump(blob, f, -1)

TILE_SIZE = 32
ICON_SIZE = 16
SHEET_SIZE = (32, 32)
SHEET_PX = 32 * 32  # 1024

SHAPE_ID = {
        'empty': 0,
        'floor': 1,
        'solid': 2,
        'ramp_e': 3,
        'ramp_w': 4,
        'ramp_s': 5,
        'ramp_n': 6,
        }


S_EMPTY =           0
S_FLOOR =           1
S_SOLID =           2
S_RAMP_E =          3
S_RAMP_W =          4
S_RAMP_S =          5
S_RAMP_N =          6


B_SUBFLOOR =        0x0001
B_FLOOR =           0x0002
B_SOLID =           0x0004
B_PART_MASK =       0x0007

B_SHAPE_MASK =      0xf000

B_OPAQUE =          0x0008
B_NON_WALKABLE =    0x0010
B_OCCUPIED =        0x0020

B_SOLID_SHAPE =     lambda s: B_SOLID | (s << 12)



BLOCK_SIDES = ('front', 'back', 'top', 'bottom')

TERRAIN_PARTS = (
        ('spot/large',      'corner/inner/se',  'corner/inner/sw'),
        ('spot/small',      'corner/inner/ne',  'corner/inner/nw'),
        ('corner/outer/nw', 'edge/n',           'corner/outer/ne'),
        ('edge/w',          'center/v0',        'edge/e'),
        ('corner/outer/sw', 'edge/s',           'corner/outer/se'),
        ('center/v1',       'center/v2',        'center/v3'),
        )

TERRAIN_CROSS_PARTS = (
        ('cross/nw',),
        ('cross/ne',),
        )

TERRAIN_PARTS2 = dict((n, (x, y))
        for y, row in enumerate(TERRAIN_PARTS)
        for x, n in enumerate(row))

TERRAIN_CROSS_PARTS2 = dict((n, (x, y))
        for y, row in enumerate(TERRAIN_CROSS_PARTS)
        for x, n in enumerate(row))


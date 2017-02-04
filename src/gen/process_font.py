import argparse
import json
import os
import struct
import sys

from PIL import Image


def build_parser():
    parser = argparse.ArgumentParser(
            description='Process a bitmap font image into a usable form.')

    parser.add_argument('--font-image-in', metavar='FILE',
            required=True,
            help='path to the original font image')
    parser.add_argument('--first-char', metavar='CODE',
            default=0x21,
            help='ASCII code of the first glyph in the font')
    parser.add_argument('--char-list', metavar='STRING',
            help='list of all characters present in the font, in order')
    parser.add_argument('--space-width', metavar='SIZE',
            default=2,
            help='width of the space character in pixels')

    parser.add_argument('--color', metavar='COLOR', default='0xffffff',
            help='font color to use in the generated image')
    parser.add_argument('--no-shadow', action='store_true',
            help='don\'t add a drop shadow in the generated image')
    parser.add_argument('--bold', action='store_true',
            help='generate a bold font')

    parser.add_argument('--font-image-out', metavar='FILE',
            required=True,
            help='path to write the processed font image')
    parser.add_argument('--font-metrics-out', metavar='FILE',
            required=True,
            help='path to write the font metrics JSON file')

    return parser


def get_glyph_boxes(img):
    # Each glyph is surrounded by a 1px margin to the top, left, and right.
    # The top-left pixel in the margin is a different color, to mark the
    # beginning of each glyph.

    assert img.mode == 'P'
    w,h = img.size

    pixels = img.getdata(0)
    # Assume top-left corner is a dot.
    dot_color = pixels[0]

    # Get the x-coordinate of each dot.
    dots = []
    for i in range(w):
        if pixels[i] == dot_color:
            dots.append(i)

    # Count the number of glyphs, then add an extra dot for the end of the
    # image.
    glyph_count = len(dots)
    dots.append(w)

    boxes = []
    for i in range(glyph_count):
        x0 = dots[i] + 1
        x1 = dots[i + 1] - 1
        y0 = 1
        y1 = h
        boxes.append((x0, y0, x1, y1))

    return boxes

def build_code_boxes(args, boxes):
    if args.char_list is None:
        first_char = int(args.first_char, 0)
        last_char = first_char + len(boxes)
        codes = range(first_char, last_char)
    else:
        first_char = min(ord(c) for c in args.char_list)
        last_char = max(ord(c) for c in args.char_list) + 1
        codes = (ord(c) for c in args.char_list)

    return {code: box for code, box in zip(codes, boxes)}

def adjust_palette(img):
    """Adjust the image's palette so that the background is black and the
    glyphs are white."""

    data = img.getdata(0)
    dot_index = data[0]
    clear_index = data[1]

    palette = img.getpalette()
    for i in range(256):
        if i == dot_index:
            palette[i * 3 : i * 3 + 3] = [255, 0, 0]
        elif i == clear_index:
            palette[i * 3 : i * 3 + 3] = [0, 0, 0]
        else:
            palette[i * 3 : i * 3 + 3] = [255, 255, 255]
    img.putpalette(palette)

def build_metrics(boxes, margin_x, margin_y, bold):
    first_char = min(boxes.keys())
    last_char = max(boxes.keys())
    num_chars = last_char - first_char + 1

    xs = [0] * num_chars
    y = 0
    widths = [0] * num_chars
    height = 0

    spans = [[first_char, first_char, 0]]
    xs2 = []
    widths2 = []

    cur_x = 0
    for index, (code, (x0,y0,x1,y1)) in enumerate(sorted(boxes.items())):
        xs[code - first_char] = cur_x
        xs2.append(cur_x)

        w = x1 - x0 + margin_x + (1 if bold else 0)
        widths[code - first_char] = w
        widths2.append(w)
        cur_x += w

        height = max(height, y1 - y0 + margin_y)

        if spans[-1][1] != code:
            spans.append([code, code, index])
        spans[-1][1] += 1

    return {
            'xs': xs,
            'y': y,
            'widths': widths,
            'height': height,
            'first_char': first_char,
            'spans': spans,
            'xs2': xs2,
            'widths2': widths2,
            }

def build_mask(src, code_boxes, metrics, bold):
    out_w = metrics['xs2'][-1] + metrics['widths2'][-1]
    out_h = metrics['height']

    out = Image.new('L', (out_w, out_h))

    for low, high, base_index in metrics['spans']:
        for code in range(low, high):
            idx = base_index + code - low
            x_pos = metrics['xs2'][idx]
            glyph = src.crop(code_boxes[code])
            out.paste(glyph, (x_pos, 0))

            if bold:
                out.paste(glyph, (x_pos + 1, 0), glyph.convert('L'))

    return out

def main():
    parser = build_parser()
    args = parser.parse_args()

    text_color = tuple(struct.pack('>I', int(args.color, 0))[1:]) + (255,)
    if not args.no_shadow:
        offsets = [(1, 1), (1, 0), (0, 1), (0, 0)]
        mx, my = (1, 1)
    else:
        offsets = [(0, 0)]
        mx, my = (0, 0)

    img = Image.open(args.font_image_in)

    boxes = get_glyph_boxes(img)
    code_boxes = build_code_boxes(args, boxes)
    adjust_palette(img)
    metrics = build_metrics(code_boxes, mx, my, args.bold)
    mask = build_mask(img, code_boxes, metrics, args.bold)

    out = Image.new('RGBA', mask.size, (0, 0, 0, 0))
    for offset in offsets:
        if offset == (0, 0):
            color = text_color
        else:
            color = (64, 64, 64, 255)
        out.paste(color, offset, mask)

    out.save(args.font_image_out)


    metrics['spacing'] = 0
    metrics['space_width'] = int(args.space_width)

    with open(args.font_metrics_out, 'w') as f:
        json.dump(metrics, f)

if __name__ == '__main__':
    main()

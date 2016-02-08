from collections import namedtuple
import json
import os
import sys

from PIL import Image

Input = namedtuple('Input', ('name', 'img', 'metrics'))

def main(out_img_path, out_metrics_path, args):
    inputs = []
    for i in range(0, len(args), 2):
        name, _ = os.path.splitext(os.path.basename(args[i]))
        img = Image.open(args[i + 0])
        with open(args[i + 1]) as f:
            metrics = json.load(f)
        inputs.append(Input(name, img, metrics))

    w = max(i.img.size[0] for i in inputs)
    h = sum(i.img.size[1] for i in inputs)
    out_img = Image.new('RGBA', (w, h))
    out_metrics = {}

    y = 0
    for i in inputs:
        print('place %s at %d' % (i.name, y))
        out_img.paste(i.img, (0, y))
        i.metrics['y'] = y
        out_metrics[i.name] = i.metrics

        y += i.img.size[1]

    out_img.save(out_img_path)
    with open(out_metrics_path, 'w') as f:
        json.dump(out_metrics, f)

if __name__ == '__main__':
    out_img, out_metrics = sys.argv[1:3]
    inputs = sys.argv[3:]
    main(out_img, out_metrics, inputs)

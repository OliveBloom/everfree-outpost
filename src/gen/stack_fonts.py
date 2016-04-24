from collections import namedtuple
import json
import os
import sys
import textwrap
import time

from PIL import Image

Input = namedtuple('Input', ('name', 'img', 'metrics'))

def build_rust(metrics):
    result = ''

    now = time.strftime('%Y-%m-%d %H:%M:%S')
    result += '// Generated %s by %s\n' % (now, sys.argv[0])

    result += textwrap.dedent('''
        #![crate_name = "client_fonts"]
        #![no_std]
        //! This auto-generated library defines the font metrics for each
        //! supported font, so they can be referred to by name.

        pub struct CharSpan {
            pub min: u8,
            pub max: u8,
            pub base_index: u8,
        }

        pub struct FontMetrics {
            pub spans: &'static [CharSpan],
            pub y: u8,
            pub height: u8,
            pub spacing: u8,
            pub space_width: u8,
            pub xs: &'static [u16],
            pub widths: &'static [u8],
        }
    ''')

    for name, m in sorted(metrics.items()):
        spans_code = ''
        for low, high, index in m['spans']:
            body = 'min: {low}, max: {high}, base_index: {index}' \
                    .format(low=low, high=high, index=index)
            spans_code += '    CharSpan {{ {} }},\n'.format(body)

        xs_code = ''
        widths_code = ''
        for i in range(0, len(m['xs2']), 16):
            xs_line = ' '.join('{:3d},'.format(x) for x in m['xs2'][i : i + 16])
            widths_line = ' '.join('{:3d},'.format(w) for w in m['widths2'][i : i + 16])
            xs_code += '    {}\n'.format(xs_line)
            widths_code += '    {}\n'.format(widths_line)

        result += textwrap.dedent('''

            pub const {name}_SPANS: [CharSpan; {num_spans}] = [
            {spans_code}
            ];
            pub const {name}_XS: [u16; {num_xs}] = [
            {xs_code}
            ];
            pub const {name}_WIDTHS: [u8; {num_widths}] = [
            {widths_code}
            ];
            pub const {name}: FontMetrics = FontMetrics {{
                spans: &{name}_SPANS,
                y: {y},
                height: {height},
                spacing: {spacing},
                space_width: {space_width},
                xs: &{name}_XS,
                widths: &{name}_WIDTHS,
            }};
        ''').format(
                name=name.upper(),
                num_spans=len(m['spans']),
                num_xs=len(m['xs2']),
                num_widths=len(m['widths2']),
                spans_code=spans_code,
                xs_code=xs_code,
                widths_code=widths_code,
                y=m['y'],
                height=m['height'],
                spacing=m['spacing'],
                space_width=m['space_width'],
                )

    return result


def main(out_img_path, out_metrics_path, out_rust_path, args):
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

    with open(out_rust_path, 'w') as f:
        f.write(build_rust(out_metrics))

if __name__ == '__main__':
    out_img, out_metrics, out_rust = sys.argv[1:4]
    inputs = sys.argv[4:]
    main(out_img, out_metrics, out_rust, inputs)

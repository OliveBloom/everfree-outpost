import os

from configure.template import template
from configure.util import cond, join, maybe, mk_build


def rules(i):
    return template('''
        rule render_template
            command = $python3 $root/mk/misc/render_template.py $site_config <$in >$out
            description = GEN $out
    ''', **locals())

def render_template(out_file, src_file):
    return template('''
        build %out_file: render_template %src_file $
            | $root/mk/misc/render_template.py $site_config
    ''', **locals())


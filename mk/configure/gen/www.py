import os

from configure.template import template
from configure.util import cond, join, maybe, mk_build


def rules(i):
    return template('''
        rule render_template
            command = $python3 $root/mk/misc/render_template.py $
                --in-file $in $
                --out-file $out_file $
                --img-src $root $
                --img-dir $b_www/img $
                %if not i.debug
                --optimize optipng $
                %end
                --site-config $site_config
            description = GEN $out
            depfile = $out_file.d

        rule collect_img_lists
            command = cat $in >$out
            description = GEN $out

        rule run_pandoc
            command = pandoc -f $format_in -t $format_out -o $out $in
            description = PANDOC $out
    ''', **locals())

def render_template(out_file, src_file):
    return template('''
        build %out_file %out_file-imgs.txt: render_template %src_file $
            | $root/mk/misc/render_template.py $site_config
            out_file = %out_file
    ''', **locals())

def collect_img_lists(out_file, src_files):
    return template(r'''
        build %out_file: collect_img_lists $
            %for f in src_files
            $b_www/%{f}-imgs.txt $
            %end
            %{'\n'}
    ''', **locals())

def render_markdown(out_file, src_file):
    return template('''
        build %out_file: run_pandoc %src_file
            format_in = markdown
            format_out = html5
    ''', **locals())

import os

from configure.template import template
from configure.util import join, maybe


def rules(i):
    return template('''
        rule dist_stamp
            command = touch $out
            description = STAMP $out

        copy_files = $python3 $root/mk/misc/copy_files.py $
        %if i.nix_patch_elf_loader
            --patchelf %{i.patchelf} $
            --set-elf-loader %{i.nix_patch_elf_loader}
        %end

        rule copy_file
            # Use -f to avoid "text file busy" when copying binaries
            command = $copy_files --mode file $in $out
            description = COPY $out

        rule copy_dir_stamp
            command = $copy_files --mode dir --stamp $out $copy_src $copy_dest
            description = COPY $copy_dest ($stamp)
            depfile = $stamp.d

        rule copy_list_stamp
            command = $copy_files --mode list --stamp $out $list_src $copy_dest
            description = COPY $copy_dest ($stamp)
            depfile = $stamp.d
    ''', **locals())

def read_manifest(i, path):
    contents = []
    with open(path) as f:
        s = f.read()
        s = template(s, i=i)

        for line in s.splitlines():
            line = line.strip()
            if line == '' or line[0] == '#':
                continue
            dest, _, src = line.partition(': ')

            contents.append((dest, src))
    return contents

def from_manifest(contents, manifest_stamp):
    builds = []
    def add_build(*args, **kwargs):
        builds.append(template(*args, **kwargs))

    dist_deps = []

    for dest, src in contents:
        if src.startswith('@'):
            assert dest.endswith('/'), \
                    'copy src is a file list but dest is not a directory'
            src = src[1:]
            stamp = '$builddir/dist_%s.stamp' % dest.strip('/').replace('/', '_')
            add_build('''
                build %stamp: copy_list_stamp | %src $root/mk/misc/copy_files.py
                    list_src = %src
                    copy_dest = $dist/%dest
                    stamp = %stamp
            ''', **locals())
            dist_deps.append(stamp)

        elif dest.endswith('/'):
            stamp = '$builddir/dist_%s.stamp' % dest.strip('/').replace('/', '_')
            add_build('''
                build %stamp: copy_dir_stamp | %src $root/mk/misc/copy_files.py
                    copy_src = %src
                    copy_dest = $dist/%dest
                    stamp = %stamp
            ''', **locals())
            dist_deps.append(stamp)

        else:
            add_build('''
                build $dist/%dest: copy_file %src | $root/mk/misc/copy_files.py
            ''', **locals())
            dist_deps.append('$dist/%s' % dest)

    add_build(r'''
        build $builddir/%manifest_stamp: dist_stamp | $
            %for d in dist_deps
            %{d} $
            %end
            %{'\n'}
    ''', **locals())

    return '\n\n'.join(builds)

def component(i, name):
    contents = read_manifest(i, os.path.join(i.root_dir, 'mk', '%s.manifest' % name))
    return from_manifest(contents, 'dist_component_%s.stamp' % name)

def components(i, names):
    rules = '\n\n'.join(component(i, name) for name in names)

    return rules + '\n\n' + template(r'''
        build $builddir/dist.stamp: dist_stamp | $
            %for n in names
            $builddir/dist_component_%{n}.stamp $
            %end
            %{'\n'}
    ''', **locals())

def copy(src, dest):
    return template('''
        build %dest: copy_file %src
    ''', **locals())

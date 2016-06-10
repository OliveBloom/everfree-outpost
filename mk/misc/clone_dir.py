import argparse
import os
import shutil
import stat
import sys

def build_parser():
    args = argparse.ArgumentParser()

    args.add_argument('--mode', default='dir',
            choices=('dir', 'list'),
            help='clone directory, or copy files from a list?')

    args.add_argument('src',
            help='source directory or file list')
    args.add_argument('dest',
            help='destination directory')
    args.add_argument('stamp',
            help='path to stamp file')

    return args


def read_tree(base, d, out):
    names = os.listdir(os.path.join(base, d))

    for n in names:
        path = os.path.join(d, n)

        st = os.stat(os.path.join(base, path))
        if stat.S_ISDIR(st.st_mode):
            read_tree(base, path, out)
            out[path] = None
        else:
            out[path] = st.st_mtime

def read_list(list_path, out):
    base = os.path.dirname(list_path)
    with open(list_path, 'r') as f:
        for path in f.readlines():
            path = path.strip()

            st = os.stat(os.path.join(base, path))
            if stat.S_ISDIR(st.st_mode):
                raise ValueError("can't handle directories in file list")
            else:
                out[path] = st.st_mtime


def clean_dest(base, d, src_files, seen):
    names = sorted(os.listdir(os.path.join(base, d)))

    for n in names:
        path = os.path.join(d, n)
        real_path = os.path.join(base, path)

        st = os.stat(real_path)
        is_dir = stat.S_ISDIR(st.st_mode)

        rm = False
        if path not in src_files:
            rm = True
        else:
            src_mtime = src_files[path]

            if src_mtime is None:
                if not is_dir:
                    rm = True
            else:
                if is_dir or st.st_mtime < src_mtime:
                    rm = True

        if rm:
            if not is_dir:
                #print('RM %s' % real_path)
                os.remove(real_path)
            else:
                #print('RMTREE %s' % real_path)
                shutil.rmtree(real_path)
        else:
            seen.add(path)
            if is_dir:
                clean_dest(base, path, src_files, seen)

def main():
    parser = build_parser()
    args = parser.parse_args(sys.argv[1:])
    src = args.src
    dest = args.dest
    stamp = args.stamp

    os.makedirs(dest, exist_ok=True)

    # 1) Collect all files in the source directory and their mtimes.
    src_files = {}
    if args.mode == 'dir':
        read_tree(src, '', src_files)
    else:
        read_list(src, src_files)
        # Remaining code expects src to be a directory
        src = os.path.dirname(src)

    # 2) Remove all unwanted or outdated files from the dest directory.
    present = set()
    clean_dest(dest, '', src_files, present)

    # 3) For every file missing from `dest`, copy that file from `src`.  Since
    # we previously removed outdated files, this updates those files.
    missing = set(src_files.keys()) - present

    for d in sorted(n for n in missing if src_files[n] is None):
        real_path = os.path.join(dest, d)
        #print('MKDIR %s' % real_path)
        os.mkdir(real_path)

    for f in sorted(n for n in missing if src_files[n] is not None):
        real_src_path = os.path.join(src, f)
        real_dest_path = os.path.join(dest, f)
        #print('CP %s' % real_dest_path)
        shutil.copy(real_src_path, real_dest_path)

    # 4) Create stamp and dependency files.
    with open(stamp, 'w') as f:
        pass

    with open(stamp + '.d', 'w') as f:
        f.write('%s: \\\n' % os.path.normpath(stamp))
        f.write('    %s \\\n' % src)
        for path in src_files:
            f.write('    %s \\\n' % os.path.join(src, path))
        f.write('\n\n')

if __name__ == '__main__':
    main()

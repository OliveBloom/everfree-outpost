import argparse
import os
import shutil
import stat
import subprocess
import sys

def build_parser():
    args = argparse.ArgumentParser()

    args.add_argument('--mode', default='dir',
            choices=('file', 'dir', 'list'),
            help='copy a file, a directory, or files from a list?')

    args.add_argument('--stamp', metavar='PATH',
            help='create a stamp file to indicate the copy succeeded')

    args.add_argument('--patchelf', metavar='PROG', default='patchelf',
            help='''name of the `patchelf` program''')
    args.add_argument('--set-elf-loader', metavar='LD_LINUX',
            help='''upon copying an ELF binary, clear the rpath and set the
                interpreter (loader) to LD_LINUX''')

    args.add_argument('src',
            help='source file, directory, or file list')
    args.add_argument('dest',
            help='destination file or directory')

    return args


# Functions for building the `copies` map, which maps dest path to src path (or
# to `None`, if the source is a directory).

def read_file(src, dest, out):
    out[dest] = src

def read_dir(src, dest, out):
    names = os.listdir(src)

    for n in names:
        src_path = os.path.join(src, n)
        dest_path = os.path.join(dest, n)

        out[dest_path] = src_path

        if os.path.isdir(src_path):
            read_dir(src_path, dest_path, out)

def read_list(src_list, dest, out):
    src = os.path.dirname(src_list)
    with open(src_list, 'r') as f:
        for path in f.readlines():
            path = path.strip()

            src_path = os.path.join(src, path)
            dest_path = os.path.join(dest, path)

            if os.path.isdir(src_path):
                raise ValueError("can't handle directories in file list")
            else:
                out[dest_path] = src_path


def remove(path, is_dir):
    if not is_dir:
        #print('RM %s' % real_path)
        os.remove(path)
    else:
        #print('RMTREE %s' % real_path)
        shutil.rmtree(path)

def clean_dir(dest, copies):
    '''Clean up directory `dest` by removing any files not listed in
    `copies`.'''
    names = os.listdir(dest)

    for n in names:
        dest_path = os.path.join(dest, n)
        is_dir = os.path.isdir(dest_path)

        if dest_path not in copies:
            remove(dest_path, is_dir)
        else:
            if is_dir:
                clean_dir(dest_path, copies)


def copy_file(src, dest):
    src_st = os.stat(src)
    src_dir = stat.S_ISDIR(src_st.st_mode)

    update = False

    if os.path.exists(dest):
        dest_st = os.stat(dest)
        dest_dir = stat.S_ISDIR(dest_st.st_mode)

        if src_dir != dest_dir or src_st.st_mtime > dest_st.st_mtime:
            remove(dest, dest_dir)
            update = True
    else:
        update = True

    if update:
        if src_dir:
            os.mkdir(dest)
            # The files inside will be copied individually
        else:
            shutil.copy(src, dest)

def copy_files(copies):
    for dest, src in sorted(copies.items()):
        copy_file(src, dest)


def postprocess(args, copies):
    if args.set_elf_loader is not None:
        for dest in copies:
            if os.path.isdir(dest):
                continue

            with open(dest, 'rb') as f:
                if f.read(4) != b'\x7fELF':
                    continue

            cmd = (args.patchelf,
                    '--set-interpreter', args.set_elf_loader,
                    '--remove-rpath',
                    dest)
            # Just try it and see if it works.  It will fail if the binary is
            # not dynamically linked.
            subprocess.call(cmd,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL)


def main():
    parser = build_parser()
    args = parser.parse_args(sys.argv[1:])
    src = args.src
    dest = args.dest

    # 1) Figure out which copies to perform
    copies = {}
    if args.mode == 'file':
        read_file(src, dest, copies)
    elif args.mode == 'dir':
        read_dir(src, dest, copies)
    elif args.mode == 'list':
        read_list(src, dest, copies)
    else:
        assert False, 'invalid mode: %r' % args.mode

    # 2) Remove all unwanted or outdated files from the dest directory.
    if args.mode != 'file':
        os.makedirs(dest, exist_ok=True)
        clean_dir(dest, copies)

    # 3) Copy updated files from `src` to `dest`
    copy_files(copies)

    # 4) Perform optional postprocessing
    postprocess(args, copies)

    # 5) Create stamp and dependency files.
    if args.stamp:
        stamp = args.stamp
        with open(stamp, 'w') as f:
            pass

        with open(stamp + '.d', 'w') as f:
            f.write('%s: \\\n' % os.path.normpath(stamp))
            f.write('    %s \\\n' % src)
            for path in sorted(copies.values()):
                f.write('    %s \\\n' % path)
            f.write('\n\n')

if __name__ == '__main__':
    main()

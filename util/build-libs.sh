#!/bin/bash
# Script to build the required Rust libraries.  Set up a directory with clones
# of Rust and all the libraries listed in README.md, and check out the
# specified revision of each one.  Then set $RUSTC and run this script from
# that directory.  Libraries will be placed in $PWD/lib.
set -e

RUSTC=${RUSTC:-rustc}

base=$PWD

mkdir -p $base/lib $base/src

edo() {
    echo $'\x1b[34m >>> \x1b[0m'"$@"
    "$@"
}

build_src() {
    local src=$1
    local crate=$2
    shift 2
    edo $RUSTC -L $base/lib --out-dir $base/lib --crate-type=lib "$src" \
        -O --crate-name=$crate "$@" \
        --extern libc=$base/lib/liblibc.rlib \
        --extern log=$base/lib/liblog.rlib \
        --extern rand=$base/lib/librand.rlib
    edo ln -sfn $PWD/$(dirname $src)/ $base/src/lib$crate
}

build() {
    build_src src/lib.rs "$@"
}

in_dir() {
    local dir=$1
    shift 1
    if [ -n "$ONLY_DIR" ] && [ "$dir" != "$ONLY_DIR" ]; then return 0; fi
    pushd "$dir"
    "$@"
    popd
}

in_dir libc  build libc \
    --cfg 'feature="cargo-build"' \
    --cfg 'feature="use_std"'

in_dir bitflags  build bitflags
in_dir rand  build rand
in_dir log  build log \
    --cfg 'feature="use_std"'
in_dir log/env  build env_logger
in_dir rustc-serialize  build rustc_serialize

in_dir time  build time

in_dir rust-cpython/python3-sys  build python3_sys -lpython3.4m \
    --cfg Py_LIMITED_API  --cfg Py_3_4 \
    --cfg 'py_sys_config="WITH_THREAD"'

in_dir linked-hash-map  build linked_hash_map

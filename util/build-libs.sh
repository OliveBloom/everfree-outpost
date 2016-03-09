#!/bin/bash
# Script to build the required Rust libraries.  Set up a directory with clones
# of Rust and all the libraries listed in README.md, and check out the
# specified revision of each one.  Then set $RUSTC and run this script from
# that directory.  Libraries will be placed in $PWD/lib.
set -e

RUSTC=${RUSTC:-rustc}

base=$PWD

build_src() {
    local src=$1
    local crate=$2
    shift 2
    $RUSTC -L $base/lib --out-dir $base/lib --crate-type=lib "$src" \
        -O --crate-name=$crate "$@" \
        --extern libc=$base/lib/liblibc.rlib \
        --extern log=$base/lib/liblog.rlib
}

build() {
    build_src src/lib.rs "$@"
}

in_dir() {
    local dir=$1
    shift 1
    pushd "$dir"
    "$@"
    popd
}

in_dir libc  build libc \
    --cfg 'feature="cargo-build"' \
    --cfg 'feature="use_std"'

in_dir bitflags  build bitflags
in_dir bitflags-0.1  build bitflags \
    -o $base/lib/libbitflags-0.1.rlib
in_dir rand  build rand
in_dir rust-memchr  build memchr
in_dir aho-corasick  build aho_corasick
in_dir utf8-ranges  build utf8_ranges
in_dir regex/regex-syntax  build regex_syntax
in_dir regex  build regex
in_dir log  build log \
    --cfg 'feature="use_std"'
in_dir log/env  build env_logger
in_dir rustc-serialize  build rustc_serialize

in_dir time  build time

in_dir rust-cpython/python3-sys  build python3_sys -lpython3.4m \
    --cfg Py_LIMITED_API  --cfg Py_3_4 \
    --cfg 'py_sys_config="WITH_THREAD"'

in_dir rusqlite/libsqlite3-sys  build libsqlite3_sys -lsqlite3
in_dir rusqlite  build rusqlite \
    --extern bitflags=$base/lib/libbitflags-0.1.rlib

in_dir linked-hash-map  build linked_hash_map
in_dir lru-cache  build lru_cache
in_dir vec-map  build vec_map

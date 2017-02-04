# Everfree Outpost

Dependencies:

 - rust-lang/rust 1.10.0 (development build)
 - kripken/emscripten-fastcomp 1.36.5  (Other Emscripten components
   are not required)

 - rust-lang/bitflags 41aa413 (0.5.0)
 - rust-lang/libc 0.2.8
 - rust-lang/log 95f4961 (0.3.5)
 - rust-lang/rand f872fda (0.3.14)
 - rust-lang/rustc-serialize 31b52d4 (0.3.18)
 - rust-lang/time 874cad8 (0.1.34)
 - contain-rs/linked-hash-map 53bf10a (0.0.9)
 - dgrunwald/rust-cpython 0.0.5 (`libpython3_sys` only)

 - python3
 - python3-pillow
 - python3-wand (with libmagickwand >= 6.9.3-3)
 - python3-yaml
 - libpython3.5
 - ninja
 - closure-compiler
 - yui-compressor
 - pandoc

Transitive dependencies (needed to compile some of the Rust libraries above):

 - BurntSushi/aho-corasick 0.5.1
 - BurntSushi/rust-memchr 0.1.10
 - BurntSushi/utf8-ranges 0.1.3
 - rust-lang/regex 0.1.55

The script `util/build_libs.sh` may be useful for compiling the Rust libraries.

Additional dependencies for the deployment scripts:

 - ansible
 - s3cmd

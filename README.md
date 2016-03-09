# Everfree Outpost

Dependencies:

 - rust-lang/rust 1.7.0 (development build)
 - kripken/emscripten-fastcomp 1.34.0-0-gdccd651  (Other Emscripten components
   are not required)
 - epdtry/rust-emscripten-passes eea6274
 - rust-lang/bitflags f27b6f2
 - rust-lang/rand c6a573f
 - BurntSushi/rust-memchr 0.1.10
 - BurntSushi/aho-corasick 0.5.1
 - BurntSushi/utf8-ranges 0.1.3-2-g5b186f1
 - rust-lang/regex 0.1.55-5-g82bd6a8
 - rust-lang/log 44ed095
 - rust-lang/rustc-serialize 86cee2f
 - rust-lang/time ac188f8
 - jgallagher/rusqlite 0.6.0-56-g2cb6c59
 - contain-rs/linked-hash-map 53bf10a
 - contain-rs/lru-cache 644fd4e
 - contain-rs/vec-map d274541
 - dgrunwald/rust-cpython 0.0.5-9-g162e20d (`libpython3_sys` only)
 - python3
 - python3-pillow
 - python3-yaml
 - liblua5.1
 - ninja
 - closure-compiler
 - yui-compressor

The script `util/build_libs.sh` may be useful for compiling the Rust libraries.

Additional dependencies for the deployment scripts:

 - ansible
 - s3cmd

# Everfree Outpost

Dependencies:

 - rust-lang/rust 1.14.0
 - kripken/emscripten-fastcomp 1.37.0  (Other Emscripten components
   are not required)

 - rust-lang/bitflags 0.6.0-16-g27f0536
 - rust-lang/libc 0.2.20-24-g8d8264b
 - rust-lang/log d0c2f47
 - rust-lang/rand 0.3.15-6-g4402c90
 - rust-lang/rustc-serialize 0.3.22-2-gf424df8
 - rust-lang/time 0.1.36 (6af46da)
 - contain-rs/linked-hash-map 4fbb8ea
 - dgrunwald/rust-cpython 0.0.4-170-gb6939f1

 - python3
 - python3-pillow
 - python3-yaml
 - libpython3.5
 - ninja
 - closure-compiler
 - yui-compressor
 - pandoc

The script `util/build_libs.sh` may be useful for compiling the Rust libraries.

Additional dependencies for the deployment scripts:

 - ansible
 - s3cmd

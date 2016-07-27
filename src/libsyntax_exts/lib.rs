#![crate_name = "syntax_exts"]
#![feature(plugin_registrar, rustc_private)]
#[macro_use] extern crate bitflags;
extern crate rustc;
extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::Registry;

#[macro_use] mod parser;
mod engine_part2;
mod python_class;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("engine_part2", engine_part2::engine_part2);
    reg.register_macro("define_python_class", python_class::define_python_class);
}

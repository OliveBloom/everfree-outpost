use api::PyRef;

mod data;
mod extra;
mod storage;
mod types;
mod v3;


pub fn init(module: PyRef) {
    data::init(module);
    storage::init(module);
    types::init(module);
    v3::init(module);
}

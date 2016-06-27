use api::PyRef;

mod v3;


pub fn init(module: PyRef) {
    v3::init(module);
}

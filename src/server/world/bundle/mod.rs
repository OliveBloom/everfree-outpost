
pub use self::types::*;

pub use self::builder::Builder;
pub use self::error::{Error, Result};

pub use self::import::{Importer, import_bundle, import_world};
pub use self::export::Exporter;

pub use libserver_bundle::*;

pub mod export;
pub mod import;


pub fn read_bundle<R: ::std::io::Read>(r: &mut R) -> Result<Bundle> {
    let mut v = Vec::new();
    try!(r.read_to_end(&mut v));
    let f = try!(flat::FlatView::from_bytes(&v));
    Ok(f.unflatten_bundle())
}

pub fn write_bundle<W: ::std::io::Write>(w: &mut W, b: &Bundle) -> Result<()> {
    let mut f = flat::Flat::new();
    f.flatten_bundle(b);
    try!(f.write(w));
    Ok(())
}

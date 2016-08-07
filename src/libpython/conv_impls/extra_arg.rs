use common_proto::extra_arg::{SimpleArg, ExtraArg};

use api as py;
use api::{PyBox, PyRef, PyResult};
use conv::{Pack, Unpack};


impl<'a> Unpack<'a> for SimpleArg {
    fn unpack(obj: PyRef<'a>) -> PyResult<SimpleArg> {
        if py::int::check(obj) {
            Ok(SimpleArg::Int(try!(Unpack::unpack(obj))))
        } else if py::unicode::check(obj) {
            Ok(SimpleArg::Str(try!(Unpack::unpack(obj))))
        } else {
            pyraise!(type_error, "expected int or str");
        }
    }
}

impl<'a> Unpack<'a> for ExtraArg {
    fn unpack(obj: PyRef<'a>) -> PyResult<ExtraArg> {
        if py::int::check(obj) {
            Ok(ExtraArg::Int(try!(Unpack::unpack(obj))))
        } else if py::unicode::check(obj) {
            Ok(ExtraArg::Str(try!(Unpack::unpack(obj))))
        } else if py::dict::check(obj) {
            Ok(ExtraArg::Map(try!(Unpack::unpack(obj))))
        } else if py::list::check(obj) {
            Ok(ExtraArg::List(try!(Unpack::unpack(obj))))
        } else {
            pyraise!(type_error, "expected one of int, str, dict, list");
        }
    }
}

impl Pack for SimpleArg {
    fn pack(self) -> PyResult<PyBox> {
        match self {
            SimpleArg::Int(i) => Pack::pack(i),
            SimpleArg::Str(s) => Pack::pack(s),
        }
    }
}

impl Pack for ExtraArg {
    fn pack(self) -> PyResult<PyBox> {
        match self {
            ExtraArg::Int(i) => Pack::pack(i),
            ExtraArg::Str(s) => Pack::pack(s),
            ExtraArg::Map(h) => Pack::pack(h),
            ExtraArg::List(v) => Pack::pack(v),
        }
    }
}

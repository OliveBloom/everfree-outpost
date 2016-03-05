use python as py;
use python::{PyRef, PyResult};

use super::{Pack, Unpack};

pub fn call_method<A, R>(obj: PyRef, name: &str, args: A) -> PyResult<R>
        where A: Pack, R: for<'a> Unpack<'a> {
    let method = try!(py::object::get_attr_str(obj, name));
    let py_args = try!(Pack::pack(args));
    let py_result = try!(py::object::call(method.borrow(), py_args.borrow(), None));
    let result = try!(Unpack::unpack(py_result.borrow()));
    Ok(result)
}

pub fn call_builtin<A, R>(name: &str, args: A) -> PyResult<R>
        where A: Pack, R: for<'a> Unpack<'a> {
    let builtins = py::eval::get_builtins();
    call_method(builtins.borrow(), name, args)
}

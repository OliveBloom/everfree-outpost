use std::path::Path;

use python3_sys::*;

use api::*;


pub fn eval(code: &str) -> PyResult<PyBox> {
    let builtins = eval::get_builtins();
    let eval = try!(dict::get_item_str(builtins.borrow(), "eval"))
        .expect("missing `eval` in `__builtins__`");
    let code_obj = try!(unicode::from_str(code));
    let globals = try!(dict::new());
    let locals = try!(dict::new());
    let args = try!(tuple::pack3(code_obj, globals, locals));
    object::call(eval, args.borrow(), None)
}

pub fn exec(code: &str) -> PyResult<PyBox> {
    let builtins = eval::get_builtins();
    let exec = try!(dict::get_item_str(builtins.borrow(), "exec"))
        .expect("missing `exec` in `__builtins__`");
    let code_obj = try!(unicode::from_str(code));
    let globals = try!(dict::new());
    let locals = try!(dict::new());
    let args = try!(tuple::pack3(code_obj, globals, locals));
    object::call(exec, args.borrow(), None)
}

pub fn run_file(path: &Path) -> PyResult<()> {
    let builtins = eval::get_builtins();
    let compile = try!(dict::get_item_str(builtins.borrow(), "compile"))
        .expect("missing `compile` in `__builtins__`");
    let exec = try!(dict::get_item_str(builtins.borrow(), "exec"))
        .expect("missing `exec` in `__builtins__`");

    // Compile this little runner program to a code object.  The runner does the actual work of
    // opening and reading the indicated file.
    let runner = try!(unicode::from_str(r#"if True:  # indentation hack
        import sys
        dct = sys.modules['__main__'].__dict__
        dct['__file__'] = filename
        with open(filename, 'r') as f:
            code = compile(f.read(), filename, 'exec')
            exec(code, dct, dct)
        "#));
    let compile_args = try!(tuple::pack3(runner,
                                         try!(unicode::from_str("<runner>")),
                                         try!(unicode::from_str("exec"))));
    let runner_code = try!(object::call(compile, compile_args.borrow(), None));

    // Now `exec` the compiled runner.  We don't call `exec` directly on `runner` because `exec`
    // doesn't allow for setting the filename.
    let globals = try!(dict::new());
    let locals = try!(dict::new());
    // TODO: be smarter about non-UTF8 Path encodings
    try!(dict::set_item_str(locals.borrow(),
                            "filename",
                            try!(unicode::from_str(path.to_str().unwrap())).borrow()));
    let args = try!(tuple::pack3(runner_code, globals, locals));
    try!(object::call(exec, args.borrow(), None));
    Ok(())
}

pub fn import(name: &str) -> PyResult<PyBox> {
    let name_obj = try!(unicode::from_str(name));
    unsafe {
        PyBox::new(PyImport_Import(name_obj.unwrap()))
    }
}


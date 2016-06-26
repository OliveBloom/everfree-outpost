
#[macro_export]
macro_rules! pyexc {
    ($ty:ident, $($msg_parts:tt)*) => {
        $crate::exc::PyExc::new($crate::api::exc::$ty(),
                                format!($($msg_parts)*))
    };
}

#[macro_export]
macro_rules! pyraise {
    ($ty:ident, $($msg_parts:tt)*) => {
        return Err(Box::new(pyexc!($ty, $($msg_parts)*)))
    };
}

/// Check that a condition holds.  If it doesn't, fetch and propagate the current exception.
///
/// If no condition is provided, `pycheck!()` checks that no exception is pending.
#[macro_export]
macro_rules! pycheck {
    () => {
        pycheck!(!$crate::api::err::occurred())
    };
    ($cond:expr) => {
        if !$cond {
            return Err(Box::new($crate::api::err::fetch()));
        }
    };
}

#[macro_export]
macro_rules! pyassert {
    ($cond:expr) => {
        pyassert!($cond, runtime_error)
    };
    ($cond:expr, $exc_ty:ident) => {
        pyassert!($cond,
                  $exc_ty,
                  concat!(file!(), ": assertion failed: `", stringify!($cond), "`"))
    };
    ($cond:expr, $exc_ty:ident, $msg:expr) => {
        if !$cond {
            pyraise!($exc_ty, $msg);
        }
    };
    ($cond:expr, $exc_ty:ident, $msg:expr, $($msg_args:tt)*) => {
        if !$cond {
            pyraise!($exc_ty, $msg, $($msg_args)*);
        }
    };
}

#[macro_export]
macro_rules! pyunwrap {
    ($opt:expr) => {
        pyunwrap!($opt, runtime_error)
    };
    ($opt:expr, $exc_ty:ident) => {
        pyunwrap!($opt,
                  $exc_ty,
                  concat!(file!(), ": `", stringify!($opt), "` produced `None`"))
    };
    ($opt:expr, $exc_ty:ident, $msg:expr) => {
        match $opt {
            Some(x) => x,
            None => pyraise!($exc_ty, $msg),
        }
    };
    ($opt:expr, $exc_ty:ident, $msg:expr, $($msg_args:tt)*) => {
        match $opt {
            Some(x) => x,
            None => pyraise!($exc_ty, $msg, $($msg_args)*),
        }
    };
}

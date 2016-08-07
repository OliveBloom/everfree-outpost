//! Stub implementation of `std::error`.

use fmt::{Debug, Display};
use marker::Reflect;

pub trait Error: Debug + Display + Reflect {
    fn description(&self) -> &str;

    fn cause(&self) -> Option<&Error> { None }
}

//! Stub implementation of `std::error`.

use fmt::{Debug, Display};

pub trait Error: Debug + Display {
    fn description(&self) -> &str;

    fn cause(&self) -> Option<&Error> { None }
}

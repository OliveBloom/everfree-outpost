pub trait Param<T>: Copy {
    fn unwrap(self) -> T;
}

impl<T: Copy> Param<T> for T {
    fn unwrap(self) -> Self {
        self
    }
}

#[macro_export]
macro_rules! static_param {
    ($Name:ident : $Ty:ty = $val:expr) => {
        #[derive(Clone, Copy, Debug)]
        struct $Name;

        impl $crate::param::Param<$Ty> for $Name {
            fn unwrap(self) -> $Ty {
                $val
            }
        }
    };
}


pub trait RefParam<T> {
    fn get(&self) -> &T;
}

impl<T> RefParam<T> for T {
    fn get(&self) -> &T {
        self
    }
}

#[macro_export]
macro_rules! static_ref_param {
    ($Name:ident : $Ty:ty = $val:expr) => {
        #[derive(Clone, Copy, Debug)]
        struct $Name;

        impl $crate::param::RefParam<$Ty> for $Name {
            fn get(&self) -> &$Ty {
                static VAL: $Ty = $val;
                &$val
            }
        }
    };
}


#[macro_export]
macro_rules! static_val_ref_param {
    ($Name:ident : $Ty:ty = $val:expr) => {
        #[derive(Clone, Copy, Debug)]
        struct $Name;

        impl $crate::param::Param<$Ty> for $Name {
            fn unwrap(self) -> $Ty {
                $val
            }
        }

        impl $crate::param::RefParam<$Ty> for $Name {
            fn get(&self) -> &$Ty {
                static VAL: $Ty = $val;
                &$val
            }
        }
    };
}

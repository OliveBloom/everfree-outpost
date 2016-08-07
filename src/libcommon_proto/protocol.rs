macro_rules! protocol {
    (
        protocol $Proto:ident [$op:ident :: $Opcode:ident = $optype:ty] {
            $(
                [$code:expr] $Op:ident { $($argname:ident : $argty:ty),* },
            )*
        }
    ) => {

        mod $op {
            use $crate::wire;
            use std::io::{self, Read, Write};

            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            pub struct $Opcode(pub $optype);

            impl From<$Opcode> for $optype {
                fn from(op: $Opcode) -> $optype {
                    op.0
                }
            }

            impl wire::ReadFrom for $Opcode {
                fn read_from<R: Read>(r: &mut R) -> io::Result<$Opcode> {
                    let raw = try!(wire::ReadFrom::read_from(r));
                    Ok($Opcode(raw))
                }
            }

            impl wire::WriteTo for $Opcode {
                fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
                    self.0.write_to(w)
                }
            }

            impl wire::Size for $Opcode {
                fn size(&self) -> usize {
                    self.0.size()
                }
            }

            $(
                #[allow(non_upper_case_globals, dead_code)]
                pub const $Op: $Opcode = $Opcode($code);
            )*
        }

        protocol_enum! {
            protocol $Proto [$op::$Opcode] {
                $( $Op { $($argname: $argty),* }, )*
            }
        }
    };
}

macro_rules! protocol_enum {
    (
        protocol $Proto:ident [$op:ident :: $Opcode:ident] {
            $(
                $Op:ident { $($argname:ident : $argty:ty),* },
            )*
        }
    ) => {
        #[derive(Debug)]
        pub enum $Proto {
            $( $Op($($argty,)*), )*
        }

        impl $crate::wire::ReadFrom for $Proto {
            fn read_from<R: ::std::io::Read>(r: &mut R) -> ::std::io::Result<$Proto> {
                let op = try!($crate::wire::ReadFrom::read_from(r));
                match op {
                    $(
                        $op::$Op => {
                            let ($($argname,)*) = try!($crate::wire::ReadFrom::read_from(r));
                            Ok($Proto::$Op($($argname,)*))
                        },
                    )*
                    _ => fail!("unrecognized opcode: {:?}", op),
                }
            }
        }

        impl $crate::wire::WriteTo for $Proto {
            fn write_to<W: ::std::io::Write>(&self, w: &mut W) -> ::std::io::Result<()> {
                match *self {
                    $(
                        $Proto::$Op($(ref $argname,)*) =>
                            $crate::wire::WriteTo::write_to(&($op::$Op, $($argname,)*), w),
                    )*
                }
            }
        }

        impl $crate::wire::Size for $Proto {
            fn size(&self) -> usize {
                match *self {
                    $(
                        $Proto::$Op($(ref $argname,)*) =>
                            $crate::wire::Size::size(&($op::$Op, $($argname,)*)),
                    )*
                }
            }
        }
    };
}

use std::prelude::v1::*;


protocol! {
    protocol Request [request_op::Opcode = u16] {
        [0xff00] AddClient { wire: u16, flags: u32, name: String },
        [0xff01] RemoveClient { wire: u16 },
        [0xff03] ReplCommand { cookie: u16, cmd: String },
        [0xff05] Shutdown { __: () },
        [0xff06] RestartServer { __: () },
        [0xff07] RestartClient { __: () },
        [0xff08] RestartBoth { __: () },
    }
}

protocol! {
    protocol Response [response_op::Opcode = u16] {
        [0xff02] ClientRemoved { wire: u16 },
        [0xff04] ReplResult { cookie: u16, msg: String },
    }
}

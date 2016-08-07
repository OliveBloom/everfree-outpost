//! Small functions that need to run on background threads.  Currently this just means
//! serialization and deserialization of requests/responses.
//!
//! De/serialization is actually pretty fast, but the input side does need to run in a separate
//! thread so the main `Engine` loop can `select` over a channel of incoming `Input`s along with
//! the channels for other types of events.

use std::io::{self, Read, Write, Cursor};
use std::iter;
use std::sync::mpsc::{Sender, Receiver};
use std::u16;

use libcommon_proto::{game, control};
use libcommon_proto::wire::{ReadFrom, WriteTo, Size};

use types::WireId;
use util::bytes::{ReadBytes, WriteBytes};


#[derive(Debug)]
pub enum Input {
    Control(control::Request),
    Game(WireId, game::Request),
}

fn read_one<R: Read>(r: &mut R) -> io::Result<Input> {
    let (raw_id, len) = try!(r.read_bytes::<(u16, u16)>());
    let mut buf = iter::repeat(0).take(len as usize).collect::<Vec<_>>();
    try!(r.read_exact(&mut buf));

    if raw_id == 0 {
        let msg = try!(control::Request::read_from(&mut Cursor::new(&buf)));
        Ok(Input::Control(msg))
    } else {
        let msg = try!(game::Request::read_from(&mut Cursor::new(&buf)));
        Ok(Input::Game(WireId(raw_id), msg))
    }
}

pub fn run_input<R: Read>(mut r: R, send: Sender<Input>) -> io::Result<()> {
    loop {
        match read_one(&mut r) {
            Ok(input) => {
                trace!("IN: {:?}", input);
                send.send(input).unwrap();
            }
            Err(e) => {
                use std::io::ErrorKind::*;
                warn!("error reading message from wire: {}", e);
                match e.kind() {
                    NotFound |
                    PermissionDenied |
                    ConnectionRefused |
                    ConnectionReset |
                    ConnectionAborted |
                    NotConnected |
                    BrokenPipe => return Err(e),
                    _ => {},
                }
            }
        }
    }
}


#[derive(Debug)]
pub enum Output {
    Control(control::Response),
    Game(WireId, game::Response),
}

fn write_one<W: Write>(w: &mut W, output: Output) -> io::Result<()> {
    match output {
        Output::Control(msg) => {
            let size = msg.size();
            assert!(size <= u16::MAX as usize);

            try!(w.write_bytes((0_u16, size as u16)));
            let mut buf = Vec::with_capacity(size);
            try!(msg.write_to(&mut buf));
            try!(w.write_all(&buf));
        },

        Output::Game(wire_id, msg) => {
            let size = msg.size();
            assert!(size <= u16::MAX as usize);

            try!(w.write_bytes((wire_id.unwrap(), size as u16)));
            let mut buf = Vec::with_capacity(size);
            try!(msg.write_to(&mut buf));
            try!(w.write_all(&buf));
        },
    }
    try!(w.flush());
    Ok(())
}

pub fn run_output<W: Write>(mut w: W, recv: Receiver<Output>) -> io::Result<()> {
    loop {
        let output = recv.recv().unwrap();
        trace!("OUT: {:?}", output);
        try!(write_one(&mut w, output));
    }
}

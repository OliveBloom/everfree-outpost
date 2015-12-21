#![crate_name = "syntax_exts"]
#![feature(plugin_registrar, rustc_private)]
#[macro_use] extern crate bitflags;
extern crate rustc;
extern crate syntax;

use std::collections::HashMap;
use std::fmt;
use std::error::Error;

use rustc::plugin::Registry;
use syntax::ast::TokenTree;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacEager, DummyResult};
use syntax::parse;
use syntax::parse::token::{Token, DelimToken};
use syntax::util::small_vector::SmallVector;


#[derive(Clone, Debug)]
struct ParseError(String, Span);
type ParseResult<T> = Result<T, ParseError>;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        &self.0
    }
}

macro_rules! fail {
    ($sp:expr, $($args:tt)*) => {
        return Err(ParseError(format!($($args)*), $sp))
    };
}


struct Parser<'a> {
    sp: Span,
    tokens: &'a [TokenTree],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(sp: Span, tokens: &'a [TokenTree]) -> Parser<'a> {
        Parser {
            sp: sp,
            tokens: tokens,
            pos: 0,
        }
    }

    fn peek(&self) -> &'a TokenTree {
        &self.tokens[self.pos]
    }

    fn eof(&self) -> bool {
        self.pos == self.tokens.len()
    }

    fn expected<T>(&self, what: &str) -> ParseResult<T> {
        if self.eof() {
            fail!(self.sp, "expected {}, but saw end of input", what)
        } else {
            let t = self.peek();
            fail!(t.get_span(), "expected {}, but saw {:?}", what, t)
        }
    }

    fn take(&mut self) -> ParseResult<&'a TokenTree> {
        if self.eof() {
            return self.expected("token");
        }

        let t = self.peek();
        self.pos += 1;
        Ok(t)
    }

    fn take_eof(&self) -> ParseResult<()> {
        if !self.eof() {
            return self.expected("end of input");
        }
        Ok(())
    }

    fn take_ident(&mut self) -> ParseResult<(&'a str, Span)> {
        if self.eof() {
            return self.expected("ident");
        }

        match *self.peek() {
            TokenTree::TtToken(sp, Token::Ident(ref id, _style)) => {
                try!(self.take());
                Ok((&id.name.as_str(), sp))
            },
            _ => { return self.expected("ident"); },
        }
    }

    fn take_word(&mut self, word: &str) -> ParseResult<()> {
        if self.eof() {
            return self.expected(&format!("\"{}\"", word));
        }

        match *self.peek() {
            TokenTree::TtToken(_, Token::Ident(ref id, _style))
                    if id.name.as_str() == word => {
                try!(self.take());
                Ok(())
            },
            _ => { return self.expected(&format!("\"{}\"", word)); },
        }
    }

    fn take_exact(&mut self, token: Token) -> ParseResult<()> {
        if self.eof() {
            return self.expected(&format!("{:?}", token));
        }

        match *self.peek() {
            TokenTree::TtToken(_, ref t) if t == &token => {
                try!(self.take());
                Ok(())
            },
            _ => { return self.expected(&format!("{:?}", token)); },
        }
    }

    fn take_delimited(&mut self, delim: DelimToken) -> ParseResult<Parser<'a>> {
        if self.eof() {
            return self.expected(&format!("opening {:?}", delim));
        }

        match *self.peek() {
            TokenTree::TtDelimited(sp, ref t) if t.delim == delim => {
                try!(self.take());
                Ok(Parser::new(sp, &t.tts))
            },
            _ => { return self.expected(&format!("opening {:?}", delim)); },
        }
    }
}

macro_rules! parts {
    ($flags_name:ident :
            $(($idx:expr) $name:ident = $ty:ty;)*) => {
        bitflags! {
            flags $flags_name: u32 {
                $(
                    const $name = 1 << $idx,
                )*
            }
        }

        static PART_TYPES: &'static [&'static str] = &[
            $(stringify!($ty),)*
        ];
    };
}

parts! {
    EngineParts:
        (0) WORLD =         ::world::World<'d>;
        (1) SCRIPT =        ::script::ScriptEngine;
        (2) EXTRA =         ::logic::extra::Extra;
        (3) MESSAGES =      ::messages::Messages;
        (4) TIMER =         ::timer::Timer;
        (5) PHYSICS =       ::physics::Physics<'d>;
        (6) VISION =        ::vision::Vision;
        (7) AUTH =          ::auth::Auth;
        (8) CHUNKS =        ::chunks::Chunks<'d>;
        (9) CACHE =         ::cache::TerrainCache;
        (10) TERRAIN_GEN =  ::terrain_gen::TerrainGen<'d>;
}

const NUM_PARTS: usize = 11;

fn build_flag_map() -> HashMap<&'static str, EngineParts> {
    #![allow(non_snake_case, unused_variables)]
    let mut fm = HashMap::new();
    macro_rules! flags {
        ($($name:ident = $value:expr;)*) => {
            {
                $(
                    let $name = {
                        fm.insert(stringify!($name), $value);
                        $value
                    };
                )*
            }
        };
    }

    flags!(
        world = WORLD;
        script = SCRIPT;
        extra = EXTRA;
        messages = MESSAGES;
        timer = TIMER;
        physics = PHYSICS;
        vision = VISION;
        auth = AUTH;
        chunks = CHUNKS;
        cache = CACHE;
        terrain_gen = TERRAIN_GEN;

        VisionHooks = world | messages;
        VisionFragment = vision | VisionHooks;
        WorldHooks = world | script | timer | extra | vision | cache | VisionFragment;
        WorldFragment = world | WorldHooks;

        HiddenVisionFragment = vision;
        HiddenWorldHooks = world | script | timer | extra | cache | HiddenVisionFragment;
        HiddenWorldFragment = world | HiddenWorldHooks;

        PhysicsFragment = physics | world | cache | WorldFragment;

        TerrainGenFragment = terrain_gen | script | WorldFragment;

        SaveReadHooks = script | timer | messages | HiddenWorldFragment;
        SaveReadFragment = HiddenWorldFragment | SaveReadHooks;
        SaveWriteHooks = script | timer | messages;

        ChunkProvider = HiddenWorldFragment | SaveReadFragment | TerrainGenFragment;
        ChunksFragment = chunks | world | ChunkProvider;

        EngineRef = EngineParts::all();
    );

    fm
}

fn parse_engine_part_typedef<'a>(table: &HashMap<&'static str, EngineParts>,
                                 mut p: Parser<'a>)
                                 -> ParseResult<(bool, &'a str, EngineParts)> {
    let is_pub = p.take_word("pub").is_ok();
    let (name, name_sp) = try!(p.take_ident());

    // Support shorthand syntax: engine_part_typedef!(pub VisionHooks);
    // Meaning: engine_part_typedef!(pub VisionHooks(VisionHooks));
    let mut body = match p.take_delimited(DelimToken::Paren) {
        Ok(x) => x,
        Err(_) => {
            if !p.eof() {
                return p.expected("open paren or end of input");
            }
            if let Some(&flags) = table.get(name) {
                return Ok((is_pub, name, flags));
            } else {
                fail!(name_sp, "struct name {:?} is not a valid engine part name", name);
            }
        }
    };

    let mut flags = EngineParts::empty();

    // Whether it's okay to accept an ident in the current state.
    let mut ready = true;
    while !body.eof() {
        if !ready {
            return body.expected("comma");
        }
        ready = false;

        let (part_name, part_name_sp) = try!(body.take_ident());
        if let Some(&part_flags) = table.get(part_name) {
            flags = flags | part_flags;
        } else {
            fail!(part_name_sp, "expected engine part name, but saw {:?}", part_name);
        }

        // Try to take a comma.  On success, we can take another ident.  In either case, we can
        // take the closing paren.
        if let Ok(_) = body.take_exact(Token::Comma) {
            ready = true;
        }
    }

    try!(body.take_eof());

    Ok((is_pub, name, flags))
}

fn engine_part_typedef(cx: &mut ExtCtxt,
                       sp: Span,
                       args: &[TokenTree]) -> Box<MacResult + 'static> {
    let fm = build_flag_map();
    let p = Parser::new(sp, args);
    let (is_pub, name, flags) = match parse_engine_part_typedef(&fm, p) {
        Ok(x) => x,
        Err(e) => {
            cx.span_err(e.1, e.description());
            return DummyResult::any(sp);
        },
    };

    let mut arg_str = String::new();

    for i in 0 .. NUM_PARTS {
        if flags.bits() & (1 << i) == 0 {
            arg_str.push_str(", ()");
        } else {
            arg_str.push_str(", ");
            arg_str.push_str(PART_TYPES[i]);
        }
    }

    let source_str = format!("engine_part_typedef_{}!({}{});",
                             if is_pub { "pub" } else { "priv" },
                             name,
                             arg_str);

    let item = parse::parse_item_from_source_str(
        "<engine_part_typedef!>".to_owned(),
        source_str,
        cx.cfg.clone(),
        cx.parse_sess).unwrap();     // TODO: error handling
    MacEager::items(SmallVector::one(item))
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("engine_part_typedef", engine_part_typedef);
}

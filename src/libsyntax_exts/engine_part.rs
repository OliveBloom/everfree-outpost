use std::collections::HashMap;
use std::error::Error;

use syntax::ast::TokenTree;
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacEager, DummyResult};
use syntax::parse;
use syntax::parse::token::{Token, DelimToken};
use syntax::util::small_vector::SmallVector;

use parser::{self, Parser};

use self::parts::*;

mod parts {
    #![allow(dead_code)]

    macro_rules! parts {
        ($flags_name:ident :
                $(($idx:expr) $name:ident = $ty:ty;)*) => {
            bitflags! {
                pub flags $flags_name: u32 {
                    $(
                        const $name = 1 << $idx,
                    )*
                }
            }

            pub static PART_TYPES: &'static [&'static str] = &[
                $(stringify!($ty),)*
            ];
        };
    }

    parts! {
        EngineParts:
            (0) WORLD =         ::world::World<'d>;
            (1) EXTRA =         ::logic::extra::Extra;
            (2) MESSAGES =      ::messages::Messages;
            (3) TIMER =         ::timer::Timer;
            (4) PHYSICS =       ::physics::Physics<'d>;
            (5) VISION =        ::vision::Vision;
            (6) CHUNKS =        ::chunks::Chunks<'d>;
            (7) CACHE =         ::cache::TerrainCache;
            (8) TERRAIN_GEN =   ::terrain_gen::TerrainGen;
            (9) CHAT =         ::chat::Chat;
    }
}

const NUM_PARTS: usize = 10;

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
        extra = EXTRA;
        messages = MESSAGES;
        timer = TIMER;
        physics = PHYSICS;
        vision = VISION;
        chunks = CHUNKS;
        cache = CACHE;
        terrain_gen = TERRAIN_GEN;
        chat = CHAT;

        VisionHooks = world | messages;
        VisionFragment = vision | VisionHooks;
        WorldHooks = world | timer | extra | vision | cache | VisionFragment;
        WorldFragment = world | WorldHooks;

        HiddenVisionFragment = VisionFragment;
        HiddenWorldHooks = WorldHooks;
        HiddenWorldFragment = WorldFragment;

        PhysicsFragment = physics | world | cache | WorldFragment;

        TerrainGenFragment = terrain_gen | WorldFragment;

        ChunkProvider = HiddenWorldFragment | TerrainGenFragment;
        ChunksFragment = chunks | world | ChunkProvider;

        EngineRef = EngineParts::all();
    );

    fm
}

fn parse_engine_part_typedef<'a>(table: &HashMap<&'static str, EngineParts>,
                                 mut p: Parser<'a>)
                                 -> parser::Result<(bool, String, EngineParts)> {
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
            if let Some(&flags) = table.get(&name as &str) {
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
        if let Some(&part_flags) = table.get(&part_name as &str) {
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

pub fn engine_part_typedef(cx: &mut ExtCtxt,
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


use std::collections::HashMap;
use std::error::Error;

use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacEager, DummyResult};
use syntax::parse;
use syntax::parse::token::{Token, DelimToken};
use syntax::tokenstream::TokenTree;
use syntax::util::small_vector::SmallVector;

use parser::{self, Parser};

use self::parts::*;

mod parts {
    #![allow(dead_code)]

    macro_rules! parts {
        ($flags_name:ident :
                $(($code:ident $idx:expr) $name:ident = $ty:ty;)*) => {
            bitflags! {
                pub flags $flags_name: u32 {
                    $(
                        const $name = 1 << $idx,
                    )*
                }
            }

            pub static PART_NAMES: &'static [&'static str] = &[
                $(stringify!($name),)*
            ];

            pub static PART_TYPES: &'static [&'static str] = &[
                $(stringify!($ty),)*
            ];

            pub static PART_CODES: &'static [&'static str] = &[
                $(stringify!($code),)*
            ];
        };
    }

    parts! {
        EngineParts:
            (Wr  0) WORLD =         ::world::World<'d>;
            (Ex  1) EXTRA =         ::logic::extra::Extra;
            (Ms  2) MESSAGES =      ::messages::Messages;
            (Ti  3) TIMER =         ::timer::Timer;
            (Vi  4) VISION =        ::vision::Vision;
            (Ch  5) CHUNKS =        ::chunks::Chunks<'d>;
            (Ca  6) CACHE =         ::cache::TerrainCache;
            (Tg  7) TERRAIN_GEN =   ::terrain_gen::TerrainGen;
            (Ct  8) CHAT =          ::chat::Chat;
            (Di  9) DIALOGS =       ::dialogs::Dialogs;
            (In 10) INPUT =         ::input::Input;
            (En 11) ENERGY =        ::components::energy::Energy;
            (MP 12) MOVEMENT =      ::components::movement::Movement;
    }
}

const NUM_PARTS: usize = 13;

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
        vision = VISION;
        chunks = CHUNKS;
        cache = CACHE;
        terrain_gen = TERRAIN_GEN;
        chat = CHAT;
        dialogs = DIALOGS;
        input = INPUT;
        energy = ENERGY;
        movement = MOVEMENT;

        All = EngineParts::all();
        Components = ENERGY | MOVEMENT;
    );

    fm
}

fn parse_engine_part2<'a>(table: &HashMap<&'static str, EngineParts>,
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

fn gen_struct(name: &str, is_pub: bool, flags: EngineParts) -> String {
    let mut code = String::new();

    code.push_str(&format!("{} struct {}<'d> {{\n", if is_pub { "pub" } else { "" }, name));
    code.push_str("    _data: &'d ::data::Data,\n");
    code.push_str("    _storage: &'d ::storage::Storage,\n");
    code.push_str("    _script_hooks: &'d ::script::ScriptHooks,\n");
    code.push_str("    _now: ::types::Time,\n");
    code.push_str("    _last_tick: ::types::Time,\n");

    for i in 0 .. NUM_PARTS {
        let field_name = PART_NAMES[i].to_lowercase();
        let ty = PART_TYPES[i];

        if flags.bits() & (1 << i) == 0 {
            code.push_str(&format!("    _{}: {},\n", field_name, ty));
        } else {
            code.push_str(&format!("    pub {}: {},\n", field_name, ty));
        }
    }

    code.push_str("}\n");

    code
}

fn gen_coded_impl(name: &str, flags: EngineParts) -> String {
    let mut code = String::new();

    code.push_str(&format!("unsafe impl<'d> ::engine::split2::Coded for {}<'d> {{\n", name));

    code.push_str("    type Code = ");
    for i in 0 .. NUM_PARTS {
        if flags.bits() & (1 << i) == 0 {
            code.push_str("::engine::split2::N<");
        } else {
            code.push_str("::engine::split2::Y<");
        }
    }
    code.push_str("::engine::split2::E");
    for _ in 0 .. NUM_PARTS {
        code.push_str(">");
    }
    code.push_str(";\n");

    code.push_str("}\n");

    code
}

pub fn engine_part2(cx: &mut ExtCtxt,
                    sp: Span,
                    args: &[TokenTree]) -> Box<MacResult + 'static> {
    let fm = build_flag_map();
    let p = Parser::new(sp, args);
    let (is_pub, name, flags) = match parse_engine_part2(&fm, p) {
        Ok(x) => x,
        Err(e) => {
            cx.span_err(e.1, e.description());
            return DummyResult::any(sp);
        },
    };


    let mut items = SmallVector::zero();

    // TODO: error handling
    items.push(parse::parse_item_from_source_str(
            "<engine_part2!>".to_owned(),
            gen_struct(&name, is_pub, flags),
            cx.parse_sess).unwrap().unwrap());
    items.push(parse::parse_item_from_source_str(
            "<engine_part2!>".to_owned(),
            gen_coded_impl(&name, flags),
            cx.parse_sess).unwrap().unwrap());
    items.push(parse::parse_item_from_source_str(
            "<engine_part2!>".to_owned(),
            format!("engine_part2_impl!({});", name),
            cx.parse_sess).unwrap().unwrap());

    MacEager::items(items)
}


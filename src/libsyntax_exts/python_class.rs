use std::rc::Rc;

use syntax::ast::DUMMY_NODE_ID;
use syntax::ast::{TokenTree, Delimited};
use syntax::ast::{Ty, Ty_, DefaultBlock, SpannedIdent};
use syntax::ast::{Expr};
use syntax::ast::{Item, Item_, Mac_, Visibility};
use syntax::codemap::{Span, Spanned, DUMMY_SP};
use syntax::ext::base::{ExtCtxt, MacResult, MacEager, DummyResult};
use syntax::ext::build::AstBuilder;
use syntax::parse::PResult;
use syntax::parse::parser::Parser;
use syntax::parse::token::{self, Token, DelimToken, Nonterminal, keywords};
use syntax::ptr::P;
use syntax::util::small_vector::SmallVector;


struct ClassDef {
    name: SpannedIdent,
    ty: P<Ty>,
    type_obj: SpannedIdent,
    initializer: SpannedIdent,
    accessor: SpannedIdent,
    default_method_mac: SpannedIdent,
    members: Vec<MemberDef>,
    slots: Vec<SlotDef>,
    methods: Vec<MethodDef>,
}

struct MethodDef {
    mac: Option<SpannedIdent>,
    name: SpannedIdent,
    args: TokenTree,
    ret_ty: P<Ty>,
    body: P<Expr>,
}

struct SlotDef {
    mac: SpannedIdent,
    name: SpannedIdent,
    args: TokenTree,
    ret_ty: P<Ty>,
    body: P<Expr>,
}

struct MemberDef {
    name: SpannedIdent,
    parts: Vec<SpannedIdent>,
}

trait ParserExt<'a> {
    fn p(&mut self) -> &mut Parser<'a>;

    fn parse_word(&mut self, word: &str) -> PResult<()> {
        let p = self.p();
        let id = try!(p.parse_ident());
        if &*id.name.as_str() == word {
            Ok(())
        } else {
            Err(p.span_fatal(p.last_span,
                             &format!("expected `{}`, found `{}`", word, id.name.as_str())))
        }
    }

    fn parse_ident_span(&mut self) -> PResult<SpannedIdent> {
        let p = self.p();
        let id = try!(p.parse_ident());
        Ok(Spanned {
            node: id,
            span: p.last_span,
        })
    }

    fn parse_ret_ty2(&mut self) -> PResult<P<Ty>> {
        let p = self.p();
        if try!(p.eat(&Token::RArrow)) {
            p.parse_ty_nopanic()
        } else {
            Ok(P(Ty {
                id: DUMMY_NODE_ID,
                node: Ty_::TyTup(Vec::new()),
                span: DUMMY_SP,
            }))
        }
    }
}

impl<'a> ParserExt<'a> for Parser<'a> {
    fn p(&mut self) -> &mut Parser<'a> { self }
}

enum Mode {
    Method,
    Slot,
    Member,
}

fn parse_python_class(mut p: Parser) -> PResult<ClassDef> {
    // Header
    try!(p.parse_word("class"));
    let name = try!(p.parse_ident_span());
    try!(p.expect(&Token::Colon));
    let ty = try!(p.parse_ty_nopanic());
    try!(p.expect(&Token::OpenDelim(token::Brace)));

    // Important names
    try!(p.parse_word("type_obj"));
    let type_obj = try!(p.parse_ident_span());
    try!(p.expect(&Token::Semi));

    try!(p.parse_word("initializer"));
    let initializer = try!(p.parse_ident_span());
    try!(p.expect(&Token::Semi));

    try!(p.parse_word("accessor"));
    let accessor = try!(p.parse_ident_span());
    try!(p.expect(&Token::Semi));

    try!(p.parse_word("method_macro"));
    let method_macro = try!(p.parse_ident_span());
    try!(p.expect(&Token::Not));
    try!(p.expect(&Token::Semi));

    // Parse function/member definitions
    let mut mode = Mode::Method;
    let mut members = Vec::new();
    let mut slots = Vec::new();
    let mut methods = Vec::new();
    while p.token != Token::CloseDelim(token::Brace) {
        // Try to parse an item of the current type.
        match mode {
            Mode::Method => {
                if try!(p.eat_keyword(keywords::Fn)) {
                    methods.push(try!(parse_method(&mut p)));
                    continue;
                }
            },
            Mode::Slot => {
                if try!(p.eat_keyword(keywords::Fn)) {
                    slots.push(try!(parse_slot(&mut p)));
                    continue;
                }
            },
            Mode::Member => {
                if try!(p.eat_keyword(keywords::Let)) {
                    members.push(try!(parse_member(&mut p)));
                    continue;
                }
            },
        }

        // No item, so try to parse a mode change.
        let kw = try!(p.parse_ident_span());
        match &*kw.node.name.as_str() {
            "members" => {
                try!(p.expect(&Token::Colon));
                mode = Mode::Member;
            },
            "slots" => {
                try!(p.expect(&Token::Colon));
                mode = Mode::Slot;
            },
            "methods" => {
                try!(p.expect(&Token::Colon));
                mode = Mode::Method;
            },
            _ => {
                return Err(p.span_fatal(p.last_span,
                                        &format!("expected `members`, `slots`, or `methods`, \
                                                 found `{}`", kw.node.name.as_str())));
            },
        }
    }

    Ok(ClassDef {
        name: name,
        ty: ty,
        type_obj: type_obj,
        initializer: initializer,
        accessor: accessor,
        default_method_mac: method_macro,
        members: members,
        slots: slots,
        methods: methods,
    })
}

fn parse_method(p: &mut Parser) -> PResult<MethodDef> {
    let mac;
    if try!(p.eat(&Token::OpenDelim(token::Paren))) {
        mac = Some(try!(p.parse_ident_span()));
        try!(p.expect(&Token::Not));
        try!(p.expect(&Token::CloseDelim(token::Paren)));
    } else {
        mac = None;
    }

    let name = try!(p.parse_ident_span());
    let args = try!(p.parse_token_tree());
    let ret_ty = try!(p.parse_ret_ty2());
    let lo = p.span.lo;
    let body = try!(p.parse_block_expr(lo, DefaultBlock));

    Ok(MethodDef {
        mac: mac,
        name: name,
        args: args,
        ret_ty: ret_ty,
        body: body,
    })
}

fn parse_slot(p: &mut Parser) -> PResult<SlotDef> {
    try!(p.expect(&Token::OpenDelim(token::Paren)));
    let mac = try!(p.parse_ident_span());
    try!(p.expect(&Token::Not));
    try!(p.expect(&Token::CloseDelim(token::Paren)));

    let name = try!(p.parse_ident_span());
    let args = try!(p.parse_token_tree());
    let ret_ty = try!(p.parse_ret_ty2());
    let lo = p.span.lo;
    let body = try!(p.parse_block_expr(lo, DefaultBlock));

    Ok(SlotDef {
        mac: mac,
        name: name,
        args: args,
        ret_ty: ret_ty,
        body: body,
    })
}

fn parse_member(p: &mut Parser) -> PResult<MemberDef> {
    let name = try!(p.parse_ident_span());
    try!(p.expect(&Token::Colon));
    try!(p.expect(&Token::Eq));

    let mut parts = Vec::new();
    parts.push(try!(p.parse_ident_span()));
    while try!(p.eat(&Token::Dot)) {
        parts.push(try!(p.parse_ident_span()));
    }
    try!(p.expect(&Token::Semi));

    Ok(MemberDef {
        name: name,
        parts: parts,
    })
}


struct Builder {
    tts: Vec<TokenTree>,
}

impl Builder {
    fn new() -> Builder {
        Builder {
            tts: Vec::new(),
        }
    }

    fn emit(&mut self, tt: TokenTree) {
        self.tts.push(tt);
    }

    fn token(&mut self, token: Token, sp: Option<Span>) {
        self.emit(TokenTree::TtToken(sp.unwrap_or(DUMMY_SP), token));
    }

    fn delimited(&mut self, child: Builder, delim: DelimToken) {
        let d = Rc::new(Delimited {
            delim: delim,
            open_span: DUMMY_SP,
            tts: child.tts,
            close_span: DUMMY_SP,
        });
        self.emit(TokenTree::TtDelimited(DUMMY_SP, d));
    }

    fn ident(&mut self, id: SpannedIdent) {
        self.token(Token::Ident(id.node, token::IdentStyle::Plain), Some(id.span));
    }

    fn nonterminal(&mut self, nt: Nonterminal, sp: Option<Span>) {
        self.token(Token::Interpolated(nt), sp);
    }

    fn ty(&mut self, ty: P<Ty>) {
        let sp = ty.span;
        self.nonterminal(Nonterminal::NtTy(ty), Some(sp));
    }

    fn expr(&mut self, expr: P<Expr>) {
        let sp = expr.span;
        self.nonterminal(Nonterminal::NtExpr(expr), Some(sp));
    }

    fn comma(&mut self) {
        self.token(Token::Comma, None);
    }

    fn semi(&mut self) {
        self.token(Token::Semi, None);
    }

    fn dot(&mut self) {
        self.token(Token::Dot, None);
    }

    fn ident_comma(&mut self, id: SpannedIdent) {
        self.ident(id);
        self.comma();
    }
}


fn emit_class(out: &mut Builder, cls: ClassDef) {
    out.ident_comma(cls.name);
    out.ty(cls.ty);
    out.comma();

    out.ident_comma(cls.type_obj);
    out.ident_comma(cls.initializer);
    out.ident_comma(cls.accessor);
    let default_method_mac = cls.default_method_mac;

    let mut method_out = Builder::new();
    for method in cls.methods {
        emit_method(&mut method_out, method, default_method_mac.clone());
    }
    out.delimited(method_out, token::Brace);

    let mut slot_out = Builder::new();
    for slot in cls.slots {
        emit_slot(&mut slot_out, slot);
    }
    out.delimited(slot_out, token::Brace);

    let mut member_out = Builder::new();
    for member in cls.members {
        emit_member(&mut member_out, member);
    }
    out.delimited(member_out, token::Brace);
}

fn emit_method(out: &mut Builder, method: MethodDef, default_mac: SpannedIdent) {
    out.ident_comma(method.mac.unwrap_or(default_mac.clone()));
    out.ident_comma(method.name);
    out.emit(method.args);
    out.comma();
    out.ty(method.ret_ty);
    out.comma();
    out.expr(method.body);
    out.semi();
}

fn emit_slot(out: &mut Builder, slot: SlotDef) {
    out.ident_comma(slot.mac);
    out.ident_comma(slot.name);
    out.emit(slot.args);
    out.comma();
    out.ty(slot.ret_ty);
    out.comma();
    out.expr(slot.body);
    out.semi();
}

fn emit_member(out: &mut Builder, member: MemberDef) {
    out.ident_comma(member.name);

    let mut first = true;
    for p in member.parts {
        if !first {
            out.dot();
        } else {
            first = false;
        }
        out.ident(p);
    }

    out.semi();
}


pub fn define_python_class(cx: &mut ExtCtxt,
                           sp: Span,
                           args: &[TokenTree]) -> Box<MacResult + 'static> {
    let cls = match parse_python_class(cx.new_parser_from_tts(args)) {
        Ok(x) => x,
        Err(_) => { return DummyResult::any(sp); },
    };

    let name = cls.name.clone();
    let mut builder = Builder { tts: Vec::new(), };
    emit_class(&mut builder, cls);

    let ident = cx.ident_of("define_python_class_impl");
    let path = cx.path_ident(DUMMY_SP, ident);
    let mac_ = Mac_::MacInvocTT(path, builder.tts, name.node.ctxt);
    let mac = Spanned {
        node: mac_,
        span: sp,
    };
    let item_ = Item_::ItemMac(mac);
    let item = Item {
        ident: cx.ident_of(""),
        attrs: Vec::new(),
        id: DUMMY_NODE_ID,
        node: item_,
        vis: Visibility::Public,
        span: sp,
    };

    MacEager::items(SmallVector::one(P(item)))
}

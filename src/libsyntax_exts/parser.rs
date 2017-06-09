use std::fmt;
use std::error;
use std::result;

use syntax::codemap::Span;
use syntax::parse::token::{Token, DelimToken};
use syntax::tokenstream::TokenTree;


#[derive(Clone, Debug)]
pub struct Error(pub String, pub Span);
pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.0
    }
}

macro_rules! fail {
    ($sp:expr, $($args:tt)*) => {
        return Err($crate::parser::Error(format!($($args)*), $sp))
    };
}


pub struct Parser<'a> {
    sp: Span,
    tokens: &'a [TokenTree],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(sp: Span, tokens: &'a [TokenTree]) -> Parser<'a> {
        Parser {
            sp: sp,
            tokens: tokens,
            pos: 0,
        }
    }

    pub fn peek(&self) -> &'a TokenTree {
        &self.tokens[self.pos]
    }

    pub fn eof(&self) -> bool {
        self.pos == self.tokens.len()
    }

    pub fn expected<T>(&self, what: &str) -> Result<T> {
        if self.eof() {
            fail!(self.sp, "expected {}, but saw end of input", what)
        } else {
            let t = self.peek();
            fail!(t.get_span(), "expected {}, but saw {:?}", what, t)
        }
    }

    pub fn take(&mut self) -> Result<&'a TokenTree> {
        if self.eof() {
            return self.expected("token");
        }

        let t = self.peek();
        self.pos += 1;
        Ok(t)
    }

    pub fn take_eof(&self) -> Result<()> {
        if !self.eof() {
            return self.expected("end of input");
        }
        Ok(())
    }

    pub fn take_ident(&mut self) -> Result<(String, Span)> {
        if self.eof() {
            return self.expected("ident");
        }

        match self.peek() {
            &TokenTree::Token(sp, Token::Ident(ref id)) => {
                try!(self.take());
                Ok(((&id.name.as_str() as &str).to_owned(), sp))
            },
            _ => { return self.expected("ident"); },
        }
    }

    pub fn take_word(&mut self, word: &str) -> Result<()> {
        if self.eof() {
            return self.expected(&format!("\"{}\"", word));
        }

        match *self.peek() {
            TokenTree::Token(_, Token::Ident(ref id))
                    if &id.name.as_str() as &str == word => {
                try!(self.take());
                Ok(())
            },
            _ => { return self.expected(&format!("\"{}\"", word)); },
        }
    }

    pub fn take_exact(&mut self, token: Token) -> Result<()> {
        if self.eof() {
            return self.expected(&format!("{:?}", token));
        }

        match *self.peek() {
            TokenTree::Token(_, ref t) if t == &token => {
                try!(self.take());
                Ok(())
            },
            _ => { return self.expected(&format!("{:?}", token)); },
        }
    }

    pub fn take_delimited(&mut self, delim: DelimToken) -> Result<Parser<'a>> {
        if self.eof() {
            return self.expected(&format!("opening {:?}", delim));
        }

        match *self.peek() {
            TokenTree::Delimited(sp, ref t) if t.delim == delim => {
                try!(self.take());
                Ok(Parser::new(sp, &t.tts))
            },
            _ => { return self.expected(&format!("opening {:?}", delim)); },
        }
    }
}

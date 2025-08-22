//! Tokenization.

use crate::{
    Span,
    parse::ParsingError,
    token::{LiteralNumber, LiteralString, QuoteType, Token},
};
use std::{ops::Deref, sync::Arc};

impl TokenStream {
    /// Parses a tokenizable buffer into a token stream.
    pub fn parse(source: &str) -> Result<TokenStream, ParsingError> {
        let mut tok = Tokenizer { source, index: 0 };

        let mut buf = vec![];

        loop {
            let t = (|| {
                tok.skip_whitespace()?;

                crate::token::__impl_token!(tok);

                Err(ParsingError::new(tok.index, "invalid token"))
            })()?;
            buf.push(t);
            if let Token::EOF(_) = t {
                break Ok(TokenStream {
                    buffer: Arc::new(buf),
                    index: 0,
                    source_len: tok.source.len(),
                });
            }
        }
    }
}

pub(crate) struct Tokenizer<'source> {
    source: &'source str,
    index: usize,
}
impl<'source> Deref for Tokenizer<'source> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.source[self.index..]
    }
}

impl Tokenizer<'_> {
    fn jump(&mut self, amount: usize) {
        self.index += amount.min(self.len());
    }

    /// Skips whitespace.
    fn skip_whitespace(&mut self) -> Result<(), ParsingError> {
        loop {
            self.jump(
                self.find(|c: char| !c.is_ascii_whitespace())
                    .unwrap_or(self.len()),
            );
            if self.len() == 0 {
                break;
            }
            if self.starts_with("--") {
                // Comment
                self.jump(2);
                if self.starts_with('[') {
                    tokenize_literal_string(self)?;
                } else {
                    self.jump(self.find('\n').unwrap_or(self.len()));
                }
                continue;
            }
            break;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A stream of tokens ready to be parsed.
pub struct TokenStream {
    pub(crate) buffer: Arc<Vec<Token>>,
    index: usize,
    source_len: usize,
}

impl TokenStream {
    /// Peeks ahead in the token stream by the given number of tokens.
    ///
    /// If no tokens are left, this will return [`TokenTy![EOF]`](crate::token::EOF).
    pub fn peek(&self, ahead: usize) -> Token {
        self.buffer.get(self.index + ahead).copied().unwrap_or(
            crate::token::EOF {
                span: crate::Span::new(self.source_len, self.source_len),
            }
            .into(),
        )
    }

    /// Gets the start of the span of the current token.
    pub fn char_index(&self) -> usize {
        self.peek(0).span().start
    }

    /// Skips ahead in the token stream by the given number of tokens.
    pub fn skip(&mut self, amount: usize) {
        self.index += amount;
    }
}

pub(crate) fn tokenize_literal_number(
    t: &mut Tokenizer<'_>,
) -> Result<LiteralNumber, ParsingError> {
    let mut tok = Tokenizer { ..*t };

    let start = tok.index;

    let mut found_first = false;
    let mut found_point = false;
    let mut found_exponent = false;
    if tok.starts_with("0X") || tok.starts_with("0x") {
        tok.jump(2);
        loop {
            let Some(c) = tok.chars().next() else { break };
            if !c.is_ascii_hexdigit() {
                #[cfg(not(feature = "5.1"))]
                if c == '.' {
                    found_point = true;
                    tok.jump(1);
                } else if c == 'p' || c == 'p' {
                    found_exponent = true;
                    tok.jump(1);
                }
                break;
            }
            tok.jump(1);
            found_first = true;
        }

        if found_point {
            // Decimal
            if !found_first {
                // need at least one number
                let Some(c) = tok.chars().next() else {
                    return Err(ParsingError::new(
                        tok.index,
                        "number must have at least one digit",
                    ));
                };
                if !(c.is_ascii_hexdigit()
                    || (cfg!(not(feature = "5.1")) && (c == 'p' || c == 'P')))
                {
                    return Err(ParsingError::new(
                        tok.index,
                        "number must have at least one digit",
                    ));
                }
                tok.jump(1);
                found_first = true;
            }
            loop {
                let Some(c) = tok.chars().next() else { break };
                if !c.is_ascii_hexdigit() {
                    #[cfg(not(feature = "5.1"))]
                    if c == 'p' || c == 'P' {
                        found_exponent = true;
                        tok.jump(1);
                    }
                    break;
                }
                tok.jump(1);
            }
        }

        if found_exponent {
            found_first = false;
            loop {
                let Some(c) = tok.chars().next() else { break };
                if !found_first && (c == '+' || c == '-') {
                    tok.jump(1);
                    continue;
                }
                if !c.is_ascii_hexdigit() {
                    if !found_first {
                        return Err(ParsingError::new(
                            tok.index,
                            "number exponent must have at least one digit",
                        ));
                    }
                    break;
                }
                found_first = true;
                tok.jump(1);
            }
        }
    } else {
        loop {
            let Some(c) = tok.chars().next() else { break };
            if !c.is_ascii_digit() {
                if c == '.' {
                    found_point = true;
                    tok.jump(1);
                } else if c == 'e' || c == 'E' {
                    found_exponent = true;
                    tok.jump(1);
                }
                break;
            }
            tok.jump(1);
            found_first = true;
        }

        if found_point {
            // Decimal
            if !found_first {
                // need at least one number
                let Some(c) = tok.chars().next() else {
                    return Err(ParsingError::new(
                        tok.index,
                        "number must have at least one digit",
                    ));
                };
                if !(c.is_ascii_digit() || c == 'e' || c == 'E') {
                    return Err(ParsingError::new(
                        tok.index,
                        "number must have at least one digit",
                    ));
                }
                tok.jump(1);
                found_first = true;
            }
            loop {
                let Some(c) = tok.chars().next() else { break };
                if !c.is_ascii_digit() {
                    if c == 'e' || c == 'E' {
                        found_exponent = true;
                        tok.jump(1);
                    }
                    break;
                }
                tok.jump(1);
            }
        }

        if found_exponent {
            found_first = false;
            loop {
                let Some(c) = tok.chars().next() else { break };
                if !found_first && (c == '+' || c == '-') {
                    tok.jump(1);
                    continue;
                }
                if !c.is_ascii_digit() {
                    if !found_first {
                        return Err(ParsingError::new(
                            tok.index,
                            "number exponent must have at least one digit",
                        ));
                    }
                    break;
                }
                found_first = true;
                tok.jump(1);
            }
        }
    }
    if found_first {
        *t = tok;
        Ok(LiteralNumber {
            span: Span::new(start, t.index),
        })
    } else {
        Err(ParsingError::new(tok.index, "number must have a digit"))
    }
}

impl QuoteType {
    fn close_ahead(&self, tok: &mut Tokenizer<'_>) -> (bool, usize) {
        let mut tok = Tokenizer { ..*tok };

        match self {
            QuoteType::Single => (tok.starts_with("'"), 1),
            QuoteType::Double => (tok.starts_with('"'), 1),
            QuoteType::Bracketed { eq_count } => {
                if !tok.starts_with("]") {
                    return (false, 0);
                }
                tok.jump(1);
                if tok.len() < *eq_count {
                    return (false, 0);
                }
                if !tok[..*eq_count].chars().all(|c| c == '=') {
                    return (false, 0);
                }
                tok.jump(*eq_count);
                if !tok.starts_with("]") {
                    return (false, 0);
                }
                (true, *eq_count + 2)
            }
        }
    }
}

pub(crate) fn tokenize_literal_string(
    t: &mut Tokenizer<'_>,
) -> Result<LiteralString, ParsingError> {
    let mut tok = Tokenizer { ..*t };

    let start = tok.index;
    let quote_type;
    if tok.starts_with("\"") {
        tok.jump(1);
        quote_type = QuoteType::Double;
    } else if tok.starts_with("'") {
        tok.jump(1);
        quote_type = QuoteType::Single;
    } else if tok.starts_with("[") {
        tok.jump(1);
        let eq_count = tok.find(|c| c != '=').unwrap_or(tok.len());
        tok.jump(eq_count);
        if !tok.starts_with("[") {
            return Err(ParsingError::new(
                tok.index,
                "expected bracket to close block string",
            ));
        }
        tok.jump(1);
        quote_type = QuoteType::Bracketed { eq_count };
    } else {
        return Err(ParsingError::new(tok.index, "invalid string quote"));
    }

    let inner_start = tok.index;
    let mut was_escape = false;

    loop {
        if tok.is_empty() {
            return Err(ParsingError::new(
                tok.index,
                "reached EOF before finding end of string",
            ));
        }

        if was_escape {
            was_escape = false;
        } else {
            if tok.starts_with('\\') {
                was_escape = true;
            } else if let (true, skip_len) = quote_type.close_ahead(&mut tok) {
                let inner_end = tok.index;
                tok.jump(skip_len);
                *t = tok;

                return Ok(LiteralString {
                    start_span: Span::new(start, inner_start),
                    inner_span: Span::new(inner_start, inner_end),
                    end_span: Span::new(inner_end, t.index),
                    quote_type,
                });
            }
        }

        tok.jump(1);
    }
}

#[cfg(test)]
mod test {
    use crate::tokenize::Token;

    use super::TokenStream;
    #[test]
    fn parse_numbers() {
        macro_rules! test_num {
            ($l: literal) => {{
                let stream = TokenStream::parse($l).expect(concat!("erroneous failure: ", $l));
                assert!(
                    stream.buffer.len() == 2,
                    concat!("erroneous failure: ", $l, " {:?}"),
                    stream
                );
            }};
            (! $l: literal) => {{
                let stream = TokenStream::parse($l);
                if let Ok(s) = stream {
                    assert!(
                        s.buffer.len() > 2 || !matches!(s.buffer[0], Token::LiteralNumber(_)),
                        concat!("erroneous success: ", $l, " {:?}"),
                        s
                    )
                }
            }};
            (? $l: literal) => {
                if cfg!(feature = "5.1") {
                    test_num!(!$l)
                } else {
                    test_num!($l)
                }
            };
        }
        println!("testing...");
        test_num!("000000");
        test_num!("1");
        test_num!("12");
        test_num!("1.2");
        test_num!("123456");
        test_num!("123.4");
        test_num!(!"123.4.");
        test_num!(!"123.4.5");
        test_num!(!"123ABC");
        test_num!(!"123p4");
        test_num!(!".4.5");
        test_num!(!".");
        test_num!("1.234");
        test_num!("123.456");
        test_num!("123456.");
        test_num!(".123456");
        test_num!(!"e");
        test_num!(!"E+");
        test_num!(!".e+");
        test_num!("123.e45");
        test_num!("123.e4");
        test_num!("123.456E7");
        test_num!("123.456e+789");
        test_num!("123.456E-78");
        test_num!("123.456E-7");
        test_num!("0x0000");
        test_num!("0x1234");
        test_num!("0x12AB");
        test_num!(?"0x123.4");
        test_num!(?"0x12A.B");
        test_num!(?"0x123p4");
        test_num!(?"0x12ApB");
        test_num!(?"0x123p+4");
        test_num!(?"0x12Ap-B");
        test_num!(?"0x1.23P+45");
        test_num!(?"0x1.2AP+B5");
        test_num!(?"0x.123p-A5B");
    }

    #[test]
    fn parse_strings() {
        macro_rules! test_str {
            ($l: literal) => {{
                println!("testing: {}", $l);
                let stream = TokenStream::parse($l).expect(concat!("erroneous failure: ", $l));
                assert!(
                    stream.buffer.len() == 2 && matches!(stream.buffer[0], Token::LiteralString(_)),
                    concat!("erroneous failure: ", $l, " {:?}"),
                    stream
                );
                println!("passed: {}", $l);
            }};
            (!$l: literal) => {{
                println!("testing: {}", $l);
                let s = TokenStream::parse($l);
                if let Ok(stream) = s {
                    assert!(
                        !(stream.buffer.len() == 2
                            && matches!(stream.buffer[0], Token::LiteralString(_))),
                        concat!("erroneous success: ", $l, " {:?}"),
                        stream
                    );
                }
                println!("passed: {}", $l);
            }};
        }
        println!("testing...");
        test_str!(r#" "meow" "#);
        test_str!(r#" 'meow' "#);
        test_str!(!r#" 'meow\' "#);
        test_str!(r#" 'meow\'' "#);
        test_str!(!r#" [meow] "#);
        test_str!(r#"  [[meow]] "#);
        test_str!(!r#" [[meow\]] "#);
        test_str!(r#" [[meow\]==]] "#);
        test_str!(r#" [==[$me[o]w$]==] "#);
        test_str!(!r#" [=[meow]] "#);
        test_str!(r#" [=[meow]=] "#);
        test_str!(!r#" [=[meow]==] "#);
        test_str!(r#" [==[meow]==] "#);
        test_str!(" --\"meow\" \n 'woof' ");
        test_str!(!r#" --"meow" 'woof' "#);
        test_str!(r#" --[[meow]] 'woof' "#);
        test_str!(r#" --[=[meow]=] 'woof' "#);
    }
}

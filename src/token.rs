//! Defines all tokens in Lua.

use crate::Span;

macro_rules! def_token {
	($($(#[$meta: meta])* [$($tt: tt)+] : $($name: ident $(
		as $([$keyword: ident])? $repr: literal
	)? $(|$s: ident| {$($block: tt)+})?)? $([$extname: ident] |$es: ident| {$($eblock: tt)+})?),* ) => {
		$(
			$(#[$meta])*
			$(
				#[derive(Debug, Copy, Clone, PartialEq, Eq)]
				#[allow(missing_docs)]
				/// Don't try to remember the name of this type - use [`Token![]`](crate::Token).
				pub struct $name {
					pub span: crate::Span
				}
			)? noop!{}

			$(#[$meta])*
			$(
			impl crate::parse::Parseable for $name {
				fn parse(t: &mut crate::tokenize::TokenStream) -> Result<Self, crate::parse::ParsingError> {
					let res = t.peek(0);
					let Token::$name(tok) = res else {
						return Err(crate::parse::ParsingError::new(
							res.span().start,
							concat!("expected token ", stringify!($name))
						))
					};
					t.skip(1);
					Ok(tok)
				}
			}
			)?
			$(
			impl crate::parse::Parseable for $extname {
				fn parse(t: &mut crate::tokenize::TokenStream) -> Result<Self, crate::parse::ParsingError> {
					let res = t.peek(0);
					let Token::$extname(tok) = res else {
						return Err(crate::parse::ParsingError::new(
							res.span().start,
							concat!("expected token ", stringify!($extname))
						))
					};
					t.skip(1);
					Ok(tok)
				}
			}
			)?
		 	noop!{}
		)*
		pub(crate) static KEYWORDS: &'static [&'static str] = &[
			$(
				$(#[$meta])*
				(""
				$($($(
					, $repr,
					stringify!($keyword)
				)?)?)?
				, "").1
			),*
		];
		#[macro_export]
		#[doc(hidden)]
		macro_rules! __impl_token_ {
			($self: expr) => {
				$(
					$(#[$meta])*
					{
						$(
							$(
								static _THIS: &'static str = $repr;
								if $self.starts_with(_THIS) {
									let start = $self.index;
									$self.jump(_THIS.len());
									return Ok(Token::$name(crate::token::$name {
										span: crate::Span::new(start, $self.index)
									}))
								}
							)?
							$(if let Ok(res) = ((|$s: &mut Tokenizer| {$($block)*})(&mut $self)) {
								return Ok(res);
							})?
						)?
						$(if let Ok(res) = ((|$es: &mut Tokenizer| {$($eblock)*})(&mut $self)) {
							return Ok(res);
						})?
					}
				)*
			};
		}
		pub(crate) use __impl_token_ as __impl_token;

		#[derive(Debug, Copy, Clone, PartialEq, Eq)]
		#[allow(missing_docs)]
		/// An enumeration over all Lua tokens.
		pub enum Token {
			$(
				$(#[$meta])*
				$($name($name),)?
				$($extname($extname),)?
			)*
		}

		$(
			$(#[$meta])*
			impl TryFrom<Token> for $($name)? $($extname)? {
				type Error = ();
				fn try_from(value: Token) -> Result<Self, ()> {
					let Token :: $($name)? $($extname)? ( v ) = value else { return Err(()); };
					Ok(v)
				}
			}
			$(#[$meta])*
			impl From<$($name)? $($extname)?> for Token {
				fn from(value: $($name)? $($extname)?) -> Token {
					Token :: $($name)? $($extname)? (value)
				}
			}
		)*


		impl Token {
			/// Gets the span of the token.
			pub fn span(&self) -> crate::Span {
				match self {
					$( $(#[$meta])*
						$(Token :: $name($name { span }) => *span)?
						$(Token :: $extname(e) => e.full_span())?
					),*
				}
			}
		}

		#[macro_export]
		#[doc(hidden)]
		/// Type-macro that expands to the specified token.
		macro_rules! _TokenTy {
			$(
				($($tt)+) => {$crate::token:: $($name)? $($extname)? };
				(VAR $args: tt $($tt)+) => {$crate::token::Token:: $($name)? $($extname)? $args};
			)*
		}
		pub use _TokenTy as TokenTy;
	}
}

macro_rules! noop {
    () => {};
}

def_token! {
    [do]: Do as [keyword] "do",
    [end]: End as [keyword] "end",
    [if]: If as [keyword] "if",
    [then]: Then as [keyword] "then",
    [elseif]: ElseIf as [keyword] "elseif",
    [else]: Else as [keyword] "else",
    [for]: For as [keyword] "for",
    [in]: In as [keyword] "in",
    [while]: While as [keyword] "while",
    [break]: Break as [keyword] "break",
    [repeat]: Repeat as [keyword] "repeat",
    [until]: Until as [keyword] "until",
    #[cfg(any(feature = "5.2", feature = "5.3", feature = "5.4", feature = "LuaJIT"))]
    [goto]: Goto as [keyword] "goto",
    [local]: Local as [keyword] "local",
    [function]: Function as [keyword] "function",
    [return]: Return as [keyword] "return",
    [nil]: Nil as [keyword] "nil",
    [true]: True as [keyword] "true",
    [false]: False as [keyword] "false",
    [and]: And as [keyword] "and",
    [or]: Or as [keyword] "or",
    [not]: Not as [keyword] "not",

    [number]: LiteralNumber |s| {
        Ok::<Token, ParsingError>(Token::LiteralNumber(crate::tokenize::tokenize_literal_number(s)?))
    },
    [string]: [LiteralString] |s| {
        Ok::<Token, ParsingError>(Token::LiteralString(crate::tokenize::tokenize_literal_string(s)?))
    },

    [;]: Semicolon as ";",
    [=]: Equal as "=",
    [+]: Plus as "+",
    [-]: Minus as "-",
    [*]: Asterisk as "*",
    [/]: Slash as "/",
    [^]: Carat as "^",
    [%]: Percent as "%",
    [.]: Dot as ".",
    [..]: DoubleDot as "..",
    [<]: Less as "<",
    [<=]: LessEqual as "<=",
    [>]: Greater as ">",
    [>=]: GreaterEqual as ">=",
    [==]: DoubleEqual as "==",
    [~=]: TildeEqual as "~=",
    [#]: Hashtag as "#",
    [...]: Ellipses as "...",
    [,]: Comma as ",",
    [::]: DoubleColon as "::",
    [:]: Colon as ":",
    ['(']: OpenParen as "(",
    ['[']: OpenBracket as "[",
    ['{']: OpenBrace as "{",
    [')']: CloseParen as ")",
    [']']: CloseBracket as "]",
    ['}']: CloseBrace as "}",

    #[cfg(any(feature = "5.3", feature = "5.4"))] [/ /]: DoubleSlash as "//",
    #[cfg(any(feature = "5.3", feature = "5.4"))] [|]: Pipe as "|",
    #[cfg(any(feature = "5.3", feature = "5.4"))] [~]: Tilde as "~",
    #[cfg(any(feature = "5.3", feature = "5.4"))] [&]: Ampersand as "&",
    #[cfg(any(feature = "5.3", feature = "5.4"))] [<<]: DoubleLess as "<<",
    #[cfg(any(feature = "5.3", feature = "5.4"))] [>>]: DoubleGreater as ">>",

    [EOF]: EOF |s| {
        if !s.is_empty() {
            return Err(ParsingError::new(s.index, "expected EOF"))
        }
        Ok(crate::token::EOF { span: crate::Span::new(s.index, s.index) }.into())
    },

    [$_: ident]: Name |s| {
    	let start = s.index;
        let start_char: char = s.chars().next()
        	.ok_or(ParsingError::new(s.index, "EOF when trying to parse name"))?;
        if !(start_char.is_ascii_alphabetic() || start_char == '_') {
            return Err(ParsingError::new(s.index, "name must start with alphabetic or _"));
        }
        let end = s.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(s.len());
        let string = &s[..end];
        if crate::token::KEYWORDS.contains(&string) {
            return Err(ParsingError::new(start + end, "name cannot be a keyword"));
        }
        s.jump(end);
        Ok(
            crate::token::Name { span: crate::Span::new(start, start + end) }.into()
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LiteralString {
    pub start_span: Span,
    pub inner_span: Span,
    pub end_span: Span,
    pub quote_type: QuoteType,
}

impl LiteralString {
    pub fn full_span(&self) -> Span {
        Span::new(self.start_span.start, self.end_span.end)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QuoteType {
    Single,
    Double,
    Bracketed { eq_count: usize },
}

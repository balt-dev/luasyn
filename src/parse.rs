//! Parsing constructs.
use std::collections::VecDeque;

use crate::{element::*, token::TokenTy, tokenize::TokenStream};
pub use luasyn_proc::Parseable;

/// An error showing that something went wrong during parsing.
#[derive(Debug, Copy, Clone)]
pub struct ParsingError {
    /// The error index.
    pub index: usize,
    /// The error message.
    pub message: &'static str
}

impl ParsingError {
    /// Creates a new error at the specified index.
    pub const fn new(index: usize, message: &'static str) -> Self {
        Self { index, message }
    }
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parsing error at index {}: {}", self.index, self.message)
    }
}

impl std::error::Error for ParsingError {}

/// Defines an item as a parseable element.
pub trait Parseable: std::fmt::Debug {
    /// Parses the element from the given tokenizer.
    fn parse(t: &mut TokenStream) -> Result<Self, ParsingError>
    where
        Self: Sized;
}

impl<T: Parseable> Parseable for Box<T> {
    fn parse(t: &mut TokenStream) -> Result<Self, ParsingError>
    where
        Self: Sized,
    {
        T::parse(t).map(Box::new)
    }
}

impl<T: Parseable> Parseable for Option<T> {
    fn parse(t: &mut TokenStream) -> Result<Self, ParsingError>
    where
        Self: Sized,
    {
        let cloned_t = t.clone();
        match T::parse(t) {
            Ok(v) => Ok(Some(v)),
            Err(_) => {
                *t = cloned_t;
                Ok(None)
            }
        }
    }
}

impl<T: Parseable, U: Parseable> Parseable for (T, U) {
    fn parse(tok: &mut TokenStream) -> Result<Self, ParsingError>
    where
        Self: Sized,
    {
        T::parse(tok).and_then(|t| Ok((t, U::parse(tok)?)))
    }
}

impl<T: Parseable, S: Parseable> Parseable for Separated<T, S> {
    fn parse(t: &mut TokenStream) -> Result<Self, ParsingError>
    where
        Self: Sized,
    {
        let mut list = vec![];
        let mut last = None;
        loop {
            let mut cloned_t = t.clone();
            let Ok(val) = T::parse(&mut cloned_t) else {
                break;
            };
            last = Some(val);
            let Ok(sep) = S::parse(&mut cloned_t) else {
                break;
            };
            let val = last.take().unwrap();
            *t = cloned_t;
            list.push((val, sep));
        }
        Ok(match last {
            Some(last) => Self::Trailing(SeparatedTrailing { list, last }),
            None => Self::NoTrailing(SeparatedNoTrailing { list }),
        })
    }
}

impl<T: Parseable, S: Parseable> Parseable for SeparatedNoTrailing<T, S> {
    fn parse(t: &mut TokenStream) -> Result<Self, ParsingError>
    where
        Self: Sized,
    {
        let mut list = vec![];
        loop {
            let mut cloned_t = t.clone();
            let Ok(val) = T::parse(&mut cloned_t) else {
                break;
            };
            let Ok(sep) = S::parse(&mut cloned_t) else {
                break;
            };
            *t = cloned_t;
            list.push((val, sep));
        }
        Ok(Self { list })
    }
}

impl<T: Parseable, S: Parseable> Parseable for SeparatedTrailing<T, S> {
    /// Parses the separated list while expecting a trailing member.
    fn parse(t: &mut TokenStream) -> Result<Self, ParsingError> {
        let cloned_t = t.clone();
        let res = SeparatedNoTrailing::parse(t)?;
        let ret = SeparatedTrailing {
            list: res.list,
            last: T::parse(t)?,
        };
        *t = cloned_t;
        Ok(ret)
    }
}

impl Parseable for FunctionName {
    fn parse(t: &mut TokenStream) -> Result<Self, ParsingError>
    where
        Self: Sized,
    {
        let path = SeparatedTrailing::parse(t)?;
        let mut suffix = None;
        if let TokenTy![VAR(_) :] = t.peek(0) {
            let c = <_>::parse(t)?;
            let name = <_>::parse(t)?;
            suffix = Some((c, name));
        }
        Ok(Self { path, suffix })
    }
}

impl<T: Parseable> Parseable for Vec<T> {
    fn parse(t: &mut TokenStream) -> Result<Self, ParsingError>
    where
        Self: Sized,
    {
        let mut ret = vec![];
        loop {
            let cloned_t = t.clone();
            let Ok(val) = T::parse(t) else {
                *t = cloned_t;
                break;
            };
            ret.push(val)
        }
        Ok(ret)
    }
}

impl Parseable for Exp {
    /// Parses a binary expression tree using the shunting-yard algorithm.
    fn parse(t: &mut TokenStream) -> Result<Self, ParsingError>
    where
        Self: Sized,
    {
        #[derive(Debug)]
        enum RPNValue {
            Binary(BinaryOperand),
            Atom(ExpAtom),
            Unary(UnaryOperand),
        }
        let mut output_queue = VecDeque::<RPNValue>::new();
        let mut operator_stack = Vec::<RPNValue>::new();
        enum State { UnaryAtom, Binary }
        let mut st = State::UnaryAtom;
        loop {
            // TODO: This needs to handle unaries differenly, -x ^ y is broken atm
            match st {
                State::UnaryAtom => {
                    if let Ok(unary) = UnaryOperand::parse(t) {
                        let top_prec = operator_stack.last().map_or(0, |b| match b {
                            RPNValue::Binary(b) => b.precedence(),
                            RPNValue::Unary(_) => BinaryOperand::UNARY_PRECEDENCE,
                            _ => unreachable!()
                        });
                        if BinaryOperand::UNARY_PRECEDENCE < top_prec {
                            output_queue.push_back(RPNValue::Unary(unary));
                        } else {
                            operator_stack.push(RPNValue::Unary(unary));
                        }
                        continue;
                    }
                    let atom = ExpAtom::parse(t)?;
                    output_queue.push_back(RPNValue::Atom(atom));
                    st = State::Binary;
                },
                State::Binary => {
                    let Ok(binop) = BinaryOperand::parse(t) else { break };
                    let prec = binop.precedence();
                    let top_prec = operator_stack.last().map_or(0, |b| match b {
                        RPNValue::Binary(b) => b.precedence(),
                        RPNValue::Unary(_) => BinaryOperand::UNARY_PRECEDENCE,
                        _ => unreachable!()
                    });

                    if prec < top_prec || (binop.left_assoc() && prec == top_prec) {
                        output_queue.push_back(RPNValue::Binary(binop));
                    } else {
                        operator_stack.push(RPNValue::Binary(binop));
                    }
                    st = State::UnaryAtom;
                }
            }
        }
        while let Some(op) = operator_stack.pop() {
            output_queue.push_back(op)
        }

        // Now, output_queue holds our expression in reverse polish notation
        let mut rpn_stack = Vec::new();

        while let Some(val) = output_queue.pop_front() {
            match val {
                RPNValue::Atom(at) => rpn_stack.push(Exp::Atom(at)),
                RPNValue::Unary(un) => {
                    let ex = rpn_stack.pop().unwrap();
                    rpn_stack.push(Exp::Unary(ExpUnary {
                        operand: un,
                        value: Box::new(ex)
                    }));
                }
                RPNValue::Binary(operand) => {
                    let right = rpn_stack.pop().unwrap();
                    let left = rpn_stack.pop().unwrap();
                    rpn_stack.push(Exp::Binary(ExpBinary { left: Box::new(left), operand, right: Box::new(right) }))
                }
            }
        }

        assert!(rpn_stack.len() == 1, "had {} values on the stack after calculating rpn", rpn_stack.len());

        Ok(rpn_stack.pop().unwrap())
    }
}

impl BinaryOperand {
    const UNARY_PRECEDENCE: usize = 11;

    fn precedence(&self) -> usize {
        match self {
            BinaryOperand::Or(_) => 1,
            BinaryOperand::And(_) => 2,
            BinaryOperand::Less(_) |
            BinaryOperand::LessEqual(_) |
            BinaryOperand::Greater(_) |
            BinaryOperand::GreaterEqual(_) |
            BinaryOperand::TildeEqual(_) |
            BinaryOperand::DoubleEqual(_) => 3,
            #[cfg(any(feature = "5.3", feature = "5.4"))] BinaryOperand::Pipe(_) => 4,
            #[cfg(any(feature = "5.3", feature = "5.4"))] BinaryOperand::Tilde(_) => 5,
            #[cfg(any(feature = "5.3", feature = "5.4"))] BinaryOperand::Ampersand(_) => 6,
            #[cfg(any(feature = "5.3", feature = "5.4"))]
                BinaryOperand::DoubleLess(_) |
                BinaryOperand::DoubleGreater(_) => 7,
            BinaryOperand::DoubleDot(_) => 8,
            BinaryOperand::Plus(_) | BinaryOperand::Minus(_) => 9,
            BinaryOperand::Asterisk(_) | BinaryOperand::Slash(_) | BinaryOperand::Percent(_) => 10,
            #[cfg(any(feature = "5.3", feature = "5.4"))] BinaryOperand::DoubleSlash(_) => 10,
            //
            BinaryOperand::Carat(_) => 12,
        }
    }
    fn left_assoc(&self) -> bool {
        match self {
            BinaryOperand::Carat(_) => false,
            _ => true
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tokenize::TokenStream;
    use crate::parse::Parseable;
    use crate::parse::Exp;

    #[test]
    fn binop_test() {
        macro_rules! test {
            ($lit: literal) => {
                let mut tok = TokenStream::parse($lit).expect(concat!("token stream parsing failed: ", $lit));
                let ex = Exp::parse(&mut tok).expect(concat!("test failed: ", $lit));
                eprintln!("{ex:?}")
            };
            (!$lit: literal) => {
                let mut tok = TokenStream::parse($lit).expect(concat!("token stream parsing failed: ", $lit));
                Exp::parse(&mut tok).expect_err(concat!("test erroneously succeeded: ", $lit));
            }
        }
        test!("(-2) ^ 2");
        test!("-2 ^ 2");
    }
}

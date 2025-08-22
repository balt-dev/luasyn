#![warn(clippy::pedantic, clippy::perf, missing_docs)]
//! A parsing library for Lua's syntax tree, reminiscent of [syn](https://docs.rs/syn).

#[cfg(any(
    not(any(
        feature = "5.1",
        feature = "5.2",
        feature = "5.3",
        feature = "5.4",
        feature = "LuaJIT"
    )),
    all(feature = "5.1", feature = "5.2"),
    all(feature = "5.1", feature = "5.3"),
    all(feature = "5.1", feature = "5.4"),
    all(feature = "5.1", feature = "LuaJIT"),
    all(feature = "5.2", feature = "5.3"),
    all(feature = "5.2", feature = "5.4"),
    all(feature = "5.2", feature = "LuaJIT"),
    all(feature = "5.3", feature = "5.4"),
    all(feature = "5.3", feature = "LuaJIT"),
    all(feature = "5.4", feature = "LuaJIT"),
))]
compile_error!("exactly one lua version feature must be enabled");

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// A span of a given source file.
#[allow(missing_docs)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Creates a new span.
    pub const fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "span start must be less than or equal to end");
        Self { start, end }
    }
}

pub mod element;
pub mod parse;
pub mod token;
pub mod tokenize;

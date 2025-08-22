//! Defines all elements in the Lua syntax tree.

#![allow(missing_docs)]

use crate::{parse::Parseable, token::TokenTy};

/// Represents a list of tokens separated by another kind of token, possibly with a trailing element.
#[derive(Debug, Clone)]
pub enum Separated<T, Sep> {
    Trailing(SeparatedTrailing<T, Sep>),
    NoTrailing(SeparatedNoTrailing<T, Sep>),
}

/// Represents a list of tokens separated by another kind of token, with a trailing element.
#[derive(Debug, Clone)]
pub struct SeparatedNoTrailing<T, Sep> {
    pub list: Vec<(T, Sep)>,
}

/// Represents a list of tokens separated by another kind of token, without a trailing element.
#[derive(Debug, Clone)]
pub struct SeparatedTrailing<T, Sep> {
    pub list: Vec<(T, Sep)>,
    pub last: T,
}

#[derive(Debug, Clone)]
pub struct FunctionName {
    pub path: SeparatedTrailing<TokenTy![name], TokenTy![.]>,
    pub suffix: Option<(TokenTy![:], TokenTy![name])>,
}

pub type VarList = SeparatedTrailing<Var, TokenTy![,]>;

pub type NameList = SeparatedTrailing<TokenTy![name], TokenTy![,]>;
#[cfg(feature = "5.4")]
pub type AttrNameList = SeparatedTrailing<(TokenTy![name], Attribute), TokenTy![,]>;

pub type ExpList = SeparatedTrailing<Exp, TokenTy![,]>;

#[derive(Debug, Clone, Parseable)]
pub struct Chunk {
    pub statements: Vec<(Statement, Option<TokenTy![;]>)>,
    pub last_statement: Option<(LastStatement, Option<TokenTy![;]>)>,
}

#[derive(Debug, Clone, Parseable)]
pub enum Statement {
    Assign(StAssign),
    FunctionCall(FunctionCall),
    Block(StBlock),
    WhileLoop(StWhileLoop),
    RepeatLoop(StRepeatLoop),
    IfElse(StIfElse),
    ForRangeLoop(StForRangeLoop),
    ForInLoop(StForInLoop),
    FunctionDef(StFunctionDef),
    LocalFunctionDef(StLocalFunctionDef),
    LocalInit(StLocalInit),
    #[cfg(any(feature = "5.2", feature = "5.3", feature = "5.4", feature = "LuaJIT"))]
    Goto(StGoto),
    #[cfg(any(feature = "5.2", feature = "5.3", feature = "5.4", feature = "LuaJIT"))]
    Label(StLabel),
    #[cfg(any(feature = "5.2", feature = "5.3", feature = "5.4", feature = "LuaJIT"))]
    Break(StBreak),
}

#[derive(Debug, Clone, Parseable)]
pub struct StAssign {
    pub vars: VarList,
    pub eq: TokenTy![=],
    pub exps: ExpList,
}

#[derive(Debug, Clone, Parseable)]
pub struct StBlock {
    pub kw_do: TokenTy![do],
    pub body: Chunk,
    pub kw_end: TokenTy![end],
}

#[cfg(feature = "5.4")]
#[derive(Debug, Clone, Parseable)]
pub struct Attribute {
    pub open_angle: TokenTy![<],
    pub name: TokenTy![name],
    pub close_angle: TokenTy![>],
}

#[derive(Debug, Clone, Parseable)]
pub struct StWhileLoop {
    pub kw_while: TokenTy![while],
    pub exp: Exp,
    pub kw_do: TokenTy![do],
    pub body: Chunk,
    pub kw_end: TokenTy![end],
}

#[derive(Debug, Clone, Parseable)]
pub struct StRepeatLoop {
    pub kw_repeat: TokenTy![repeat],
    pub body: Chunk,
    pub kw_until: TokenTy![until],
    pub exp: Exp,
}

#[derive(Debug, Clone, Parseable)]
pub struct StForRangeLoop {
    pub kw_for: TokenTy![for],
    pub name: TokenTy![name],
    pub eq: TokenTy![=],
    pub start: Exp,
    pub comma: TokenTy![,],
    pub stop: Exp,
    pub step: Option<(TokenTy![,], Exp)>,
    pub kw_do: TokenTy![do],
    pub body: Chunk,
    pub kw_end: TokenTy![end],
}

#[derive(Debug, Clone, Parseable)]
pub struct StForInLoop {
    pub kw_for: TokenTy![for],
    pub namelist: NameList,
    pub kw_in: TokenTy![in],
    pub explist: ExpList,
    pub kw_do: TokenTy![do],
    pub body: Chunk,
    pub kw_end: TokenTy![end],
}

#[derive(Debug, Clone, Parseable)]
pub struct StIfElse {
    pub if_branch: IfBranch,
    pub elseif_branches: Vec<ElseIfBranch>,
    pub else_branch: Option<ElseBranch>,
}

#[derive(Debug, Clone, Parseable)]
pub struct IfBranch {
    pub kw_if: TokenTy![if],
    pub condition: Exp,
    pub kw_then: TokenTy![then],
    pub body: Chunk,
}

#[derive(Debug, Clone, Parseable)]
pub struct ElseIfBranch {
    pub kw_elseif: TokenTy![elseif],
    pub condition: Exp,
    pub kw_then: TokenTy![then],
    pub body: Chunk,
}

#[derive(Debug, Clone, Parseable)]
pub struct ElseBranch {
    pub kw_else: TokenTy![else],
    pub body: Chunk,
}

#[derive(Debug, Clone, Parseable)]
pub struct FunctionCall {
    pub prefix: Box<PrefixExp>,
    pub body: FunctionCallBody,
}

#[derive(Debug, Clone, Parseable)]
pub struct FunctionCallBody {
    pub qualifier: Option<(TokenTy![:], TokenTy![name])>,
    pub args: FunctionArgs,
}

#[derive(Debug, Clone, Parseable)]
pub enum FunctionArgs {
    List(ArgList),
    Table(TableConstructor),
    String(TokenTy![string]),
}

#[derive(Debug, Clone, Parseable)]
pub struct ArgList {
    pub open_paren: TokenTy!['('],
    pub arguments: ExpList,
    pub close_paren: TokenTy![')'],
}

#[derive(Debug, Clone, Parseable)]
pub struct StFunctionDef {
    pub kw_function: TokenTy![function],
    pub name: FunctionName,
    pub body: FunctionBody,
}

#[derive(Debug, Clone, Parseable)]
pub struct StLocalFunctionDef {
    pub local: TokenTy![local],
    pub function: TokenTy![function],
    pub name: TokenTy![name],
    pub body: FunctionBody,
}

#[derive(Debug, Clone, Parseable)]
pub struct StLocalInit {
    #[cfg(not(feature = "5.4"))]
    pub names: NameList,
    #[cfg(feature = "5.4")]
    pub attrnames: AttrNameList,
    pub eq: TokenTy![=],
    pub exps: ExpList,
}

#[cfg(any(feature = "5.2", feature = "5.3", feature = "5.4", feature = "LuaJIT"))]
#[derive(Debug, Copy, Clone, Parseable)]
pub struct StGoto {
    pub kw_goto: TokenTy![goto],
    pub label: TokenTy![name],
}

#[cfg(any(feature = "5.2", feature = "5.3", feature = "5.4", feature = "LuaJIT"))]
#[derive(Debug, Copy, Clone, Parseable)]
pub struct StLabel {
    pub start_dcolon: TokenTy![::],
    pub name: TokenTy![name],
    pub end_dcolon: TokenTy![::],
}

#[derive(Debug, Clone, Parseable)]
pub enum LastStatement {
    Return(StReturn),
    #[cfg(not(any(feature = "5.2", feature = "5.3", feature = "5.4", feature = "LuaJIT")))]
    Break(StBreak),
}

#[derive(Debug, Clone, Parseable)]
pub struct StReturn {
    pub kw_return: TokenTy![return],
    pub value: ExpList,
}

#[derive(Debug, Copy, Clone, Parseable)]
pub struct StBreak {
    pub kw_break: TokenTy![break],
}

#[derive(Debug, Clone, Parseable)]
pub struct FunctionBody {
    pub open_paren: TokenTy!['('],
    pub parlist: ParList,
    pub close_paren: TokenTy![')'],
    pub block: Chunk,
    pub kw_end: TokenTy![end],
}

#[derive(Debug, Clone, Parseable)]
pub struct ParList {
    pub names: NameList,
    pub ellipses: Option<TokenTy![...]>,
}

#[derive(Debug, Clone, Parseable)]
pub enum Var {
    Name(TokenTy![name]),
    Index(VarIndex),
    Access(VarAccess),
}

#[derive(Debug, Clone, Parseable)]
pub struct VarIndex {
    pub prefix: Box<PrefixExp>,
    pub body: VarIndexBody,
}

#[derive(Debug, Clone, Parseable)]
pub struct VarIndexBody {
    pub open_bracket: TokenTy!['['],
    pub index: Exp,
    pub close_bracket: TokenTy![']'],
}

#[derive(Debug, Clone, Parseable)]
pub struct VarAccess {
    pub prefix: Box<PrefixExp>,
    pub body: VarAccessBody,
}

#[derive(Debug, Clone, Parseable)]
pub struct VarAccessBody {
    pub dot: TokenTy![.],
    pub name: TokenTy![name],
}

#[derive(Debug, Clone, Parseable)]
pub struct PrefixExp {
    pub kind: PrefixExpKind,
    pub calls: Vec<PrefixExpSuffix>,
}

#[derive(Debug, Clone, Parseable)]
pub enum PrefixExpKind {
    Name(TokenTy![name]),
    Parens(PrefixParens),
}

#[derive(Debug, Clone, Parseable)]
pub enum PrefixExpSuffix {
    FunctionCall(FunctionCallBody),
    Index(VarIndexBody),
    Access(VarAccessBody),
}

#[derive(Debug, Clone, Parseable)]
pub struct PrefixParens {
    pub open_paren: TokenTy!['('],
    pub inner: Exp,
    pub close_paren: TokenTy![')'],
}

#[derive(Debug, Clone)]
pub enum Exp {
    Atom(ExpAtom),
    Unary(ExpUnary),
    Binary(ExpBinary),
}

#[derive(Debug, Clone, Parseable)]
pub enum ExpAtom {
    Nil(TokenTy![nil]),
    True(TokenTy![true]),
    False(TokenTy![false]),
    Literal(ExpLiteral),
    Ellipses(TokenTy![...]),
    Function(ExpFunction),
    Prefix(Box<PrefixExp>),
    Table(Box<TableConstructor>),
}

#[derive(Debug, Clone)]
pub struct ExpBinary {
    pub left: Box<Exp>,
    pub operand: BinaryOperand,
    pub right: Box<Exp>,
}

#[derive(Debug, Clone)]
pub struct ExpUnary {
    pub operand: UnaryOperand,
    pub value: Box<Exp>,
}

#[derive(Debug, Copy, Clone, Parseable)]
pub enum BinaryOperand {
    Plus(TokenTy![+]),
    Minus(TokenTy![-]),
    Asterisk(TokenTy![*]),
    Slash(TokenTy![/]),
    Carat(TokenTy![^]),
    Percent(TokenTy![%]),
    DoubleDot(TokenTy![..]),
    Less(TokenTy![<]),
    LessEqual(TokenTy![<=]),
    Greater(TokenTy![>]),
    GreaterEqual(TokenTy![>=]),
    DoubleEqual(TokenTy![==]),
    TildeEqual(TokenTy![~=]),
    And(TokenTy![and]),
    Or(TokenTy![or]),

    #[cfg(any(feature = "5.3", feature = "5.4"))]
    DoubleSlash(TokenTy![/ /]),
    #[cfg(any(feature = "5.3", feature = "5.4"))]
    Pipe(TokenTy![|]),
    #[cfg(any(feature = "5.3", feature = "5.4"))]
    Tilde(TokenTy![~]),
    #[cfg(any(feature = "5.3", feature = "5.4"))]
    Ampersand(TokenTy![&]),
    #[cfg(any(feature = "5.3", feature = "5.4"))]
    DoubleLess(TokenTy![<<]),
    #[cfg(any(feature = "5.3", feature = "5.4"))]
    DoubleGreater(TokenTy![>>]),
}

#[derive(Debug, Copy, Clone, Parseable)]
pub enum UnaryOperand {
    Minus(TokenTy![-]),
    Not(TokenTy![not]),
    Hashtag(TokenTy![#]),
    #[cfg(any(feature = "5.3", feature = "5.4"))]
    Tilde(TokenTy![~]),
}

#[derive(Debug, Clone, Parseable)]
pub struct ExpFunction {
    pub kw_function: TokenTy![function],
    pub body: Box<FunctionBody>,
}

#[derive(Debug, Clone, Parseable)]
pub struct TableConstructor {
    pub open_brace: TokenTy!['{'],
    pub fields: Separated<Field, FieldSep>,
    pub close_brace: TokenTy!['}'],
}

#[derive(Debug, Copy, Clone, Parseable)]
pub enum FieldSep {
    Comma(TokenTy![,]),
    Semicolon(TokenTy![;]),
}

#[derive(Debug, Clone, Parseable)]
pub enum Field {
    ArrayLike(Exp),
    MapLike(MapField),
    Bracketed(BracketField),
}

#[derive(Debug, Clone, Parseable)]
pub struct MapField {
    pub name: TokenTy![name],
    pub eq: TokenTy![=],
    pub exp: Exp,
}

#[derive(Debug, Clone, Parseable)]
pub struct BracketField {
    pub open_bracket: TokenTy!['['],
    pub key: Exp,
    pub close_bracket: TokenTy![']'],
    pub eq: TokenTy![=],
    pub value: Exp,
}

#[derive(Debug, Copy, Clone, Parseable)]
pub enum ExpLiteral {
    Number(TokenTy![number]),
    String(TokenTy![string]),
}

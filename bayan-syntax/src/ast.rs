use crate::span::Span;
use crate::token::{FloatSuffix, IntSuffix};

/// A complete module.
#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub span: Span,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Struct(StructDecl),
    Fn(FnDecl),
    ExternFn(ExternFnDecl),
    Global(GlobalDecl),
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub span: Span,
    pub fields: Vec<StructField>,
    pub align: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub span: Span,
    pub params: Vec<Param>,
    pub ret: Type,
    pub attrs: Vec<Attr>,
    pub body: Block,
    pub is_export: bool,
    pub is_extern: bool,
}

#[derive(Debug, Clone)]
pub struct ExternFnDecl {
    pub name: String,
    pub span: Span,
    pub params: Vec<Param>,
    pub ret: Type,
    pub attrs: Vec<Attr>,
}

#[derive(Debug, Clone)]
pub struct GlobalDecl {
    pub name: String,
    pub span: Span,
    pub ty: Type,
    pub init: ConstExpr,
    pub attrs: Vec<Attr>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub span: Span,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl(VarDecl),
    LetDecl(VarDecl),
    Assign(Assign),
    ExprStmt(Expr),
    If(IfStmt),
    Loop(LoopStmt),
    Switch(SwitchStmt),
    Return(Option<Expr>, Span),
    Trap(u64, Span),
    Goto(String, Span),
    LabeledBlock(String, Block),
    Unsafe(Block),
    Asm(AsmStmt),
    Break(Option<Expr>, Span),
    Continue(Span),
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub ty: Type,
    pub init: Expr,
    pub span: Span,
    pub is_let: bool,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LoopStmt {
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SwitchStmt {
    pub discr: Expr,
    pub cases: Vec<(ConstExpr, Block)>,
    pub default: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AsmStmt {
    pub volatile: bool,
    pub template: String,
    pub outs: Vec<(String, Type)>,
    pub ins: Vec<(Expr, Type)>,
    pub clobbers: Vec<String>,
    pub span: Span,
}

/// Types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    I8, I16, I32, I64, Bool, F32, F64, Ptr, Unit,
    Struct(String),
    Array(Box<Type>, u64),
}

impl Type {
    pub fn size_bits(&self) -> u64 {
        match self {
            Type::I8 => 8, Type::I16 => 16, Type::I32 => 32, Type::I64 => 64,
            Type::Bool => 1, Type::F32 => 32, Type::F64 => 64, Type::Ptr => 64,
            Type::Unit => 0,
            Type::Struct(_) => 0,
            Type::Array(inner, n) => inner.size_bits() * n,
        }
    }

    pub fn size_bytes(&self) -> u64 {
        (self.size_bits() + 7) / 8
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Bool)
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::Bool => write!(f, "bool"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Ptr => write!(f, "ptr"),
            Type::Unit => write!(f, "unit"),
            Type::Struct(n) => write!(f, "{}", n),
            Type::Array(inner, n) => write!(f, "[{}; {}]", inner, n),
        }
    }
}

/// Constant expressions (verifiable at compile time).
#[derive(Debug, Clone)]
pub enum ConstExpr {
    Int(u64, Span),
    Float(f64, Span),
    Bool(bool, Span),
    Null(Span),
    Unary(UnaryOp, Box<ConstExpr>, Span),
    Binary(BinaryOp, Box<ConstExpr>, Box<ConstExpr>, Span),
}

/// Expressions (surface syntax).
#[derive(Debug, Clone)]
pub enum Expr {
    Ident(String, Span),
    IntLit(u64, IntSuffix, Span),
    FloatLit(f64, FloatSuffix, Span),
    BoolLit(bool, Span),
    StringLit(String, Span),
    CharLit(char, Span),
    Null(Span),

    MethodCall {
        receiver: Box<Expr>,
        op: String,
        type_args: Vec<Type>,
        args: Vec<Expr>,
        span: Span,
    },

    Unary(UnaryOp, Box<Expr>, Span),
    Binary(BinaryOp, Box<Expr>, Box<Expr>, Span),
    Cast(CastOp, Type, Box<Expr>, Span),
    Index(Box<Expr>, Box<Expr>, Span),
    Field(Box<Expr>, String, Span),
    Paren(Box<Expr>, Span),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp { Not, Neg, Clz, Ctz, Popcnt }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    AddWrap, AddTrap, AddSat, SubWrap, SubTrap, SubSat, MulWrap, MulTrap, MulSat,
    DivS, DivU, RemS, RemU,
    ShlMask, ShlTrap, ShrUMask, ShrUTrap, ShrSMask, ShrSTrap,
    And, Or, Xor,
    Eq, Ne, LtS, LtU, LeS, LeU, GtS, GtU, GeS, GeU,
    FEqO, FEqU, FNeO, FNeU, FLtO, FLtU, FLeO, FLeU, FGtO, FGtU, FGeO, FGeU,
    AndLogical, OrLogical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CastOp { Zext, Sext, Trunc, Fptosi, Sitofp, Fptou, Uitofp, Bitcast, Ptr2int, Int2ptr }

#[derive(Debug, Clone)]
pub struct Attr {
    pub name: String,
    pub value: Option<AttrValue>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AttrValue {
    Ident(String),
    Int(u64),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Ident(_, s) | Expr::IntLit(_, _, s) | Expr::FloatLit(_, _, s)
            | Expr::BoolLit(_, s) | Expr::StringLit(_, s) | Expr::CharLit(_, s)
            | Expr::Null(s) | Expr::MethodCall { span: s, .. }
            | Expr::Unary(_, _, s) | Expr::Binary(_, _, _, s)
            | Expr::Cast(_, _, _, s) | Expr::Index(_, _, s) | Expr::Field(_, _, s)
            | Expr::Paren(_, s) => *s,
        }
    }
}

impl ConstExpr {
    pub fn span(&self) -> Span {
        match self {
            ConstExpr::Int(_, s) | ConstExpr::Float(_, s) | ConstExpr::Bool(_, s)
            | ConstExpr::Null(s) | ConstExpr::Unary(_, _, s) | ConstExpr::Binary(_, _, _, s) => *s,
        }
    }
}

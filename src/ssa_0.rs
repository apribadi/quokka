//! untyped linear intermediate represenation

use crate::symbol::Symbol;

type Arity = usize;
type Index = usize;
type Label = usize;
type Local = usize;
type Value = usize;

pub struct Module {
  pub fun_defs: Box<[FunDef]>,
  pub code: Box<[Inst]>,
}

pub struct FunDef {
  pub name: Option<Symbol>,
  pub pos: usize,
  pub len: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[derive(Debug)]
pub enum Inst {
  Label(Arity, Arity),
  Pop(Index),
  Put(Index, Value),
  Const(Symbol),
  ConstBool(bool),
  ConstI64(i64),
  DefLocal(Value),
  GetLocal(Local),
  SetLocal(Local, Value),
  GetField(Value, Symbol),
  SetField(Value, Symbol, Value),
  GetIndex(Value, Value),
  SetIndex(Value, Value, Value),
  Op1(Op1, Value),
  Op2(Op2, Value, Value),
  Call(Value),
  TailCall(Value),
  Cond(Value),
  Goto(Label),
  Ret,
  Fail,
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[derive(Debug)]
pub enum Op1 {
  Neg,
  Not,
}

// NB:
//
// we normalize some comparison operators
// - x != y => not (x == y)
// - x >= y => y <= x
// - x > y => y < x

#[derive(Clone, Copy, Eq, PartialEq)]
#[derive(Debug)]
pub enum Op2 {
  Add,
  BitAnd,
  BitOr,
  BitXor,
  CmpEq,
  CmpLe,
  CmpLt,
  Div,
  Mul,
  Rem,
  Shl,
  Shr,
  Sub,
}

impl std::fmt::Display for Op1 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Neg => write!(f, "-"),
      Self::Not => write!(f, "!"),
    }
  }
}

impl std::fmt::Display for Op2 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Add => write!(f, "+"),
      Self::BitAnd => write!(f, "&"),
      Self::BitOr => write!(f, "|"),
      Self::BitXor => write!(f, "^"),
      Self::CmpEq => write!(f, "=="),
      Self::CmpLe => write!(f, "<="),
      Self::CmpLt => write!(f, "<"),
      Self::Div => write!(f, "/"),
      Self::Mul => write!(f, "*"),
      Self::Rem => write!(f, "%"),
      Self::Shl => write!(f, "<<"),
      Self::Shr => write!(f, ">>"),
      Self::Sub => write!(f, "-"),
    }
  }
}

impl std::fmt::Display for Inst {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Label(n, _) => write!(f, "LABEL {}", n),
      Self::Pop(i) => write!(f, "= POP {}", i),
      Self::Put(i, x) => write!(f, "PUT {} %{}", i, x),
      Self::Const(s) => write!(f, "= CONST {}", s),
      Self::ConstBool(p) => write!(f, "= {}", p),
      Self::ConstI64(n) => write!(f, "= {}", n),
      Self::DefLocal(x) => write!(f, "= DEF-LOCAL %{}", x),
      Self::GetLocal(v) => write!(f, "= [ %{} ]", v),
      Self::SetLocal(v, x) => write!(f, "[ %{} ] <- %{}", v, x),
      Self::GetField(x, s) => write!(f, "= %{} [ .{} ]", x, s),
      Self::SetField(x, s, y) => write!(f, "%{} [ .{} ] <- %{}", x, s, y),
      Self::GetIndex(x, y) => write!(f, "= %{} [ %{} ]", x, y),
      Self::SetIndex(x, y, z) => write!(f, "%{} [ %{} ] <- %{}", x, y, z),
      Self::Op1(op, x) => write!(f, "= {} %{}", op, x),
      Self::Op2(op, x, y) => write!(f, "= %{} {} %{}", x, op, y),
      Self::Call(x) => write!(f, "CALL %{}", x),
      Self::TailCall(x) => write!(f, "TAIL-CALL %{}", x),
      Self::Cond(x) => write!(f, "COND %{}", x),
      Self::Goto(x) => write!(f, "==> GOTO %{}", x),
      Self::Ret => write!(f, "RET"),
      Self::Fail => write!(f, "==> FAIL"),

    }
  }
}

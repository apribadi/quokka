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
  pub name: Symbol,
  pub pos: usize,
  pub len: usize,
}

#[derive(Clone, Copy)]
pub enum Inst {
  Label(Arity, Arity),
  GetArg(Index),
  SetOut(Index, Value),
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
  Jump(Label),
  Ret,
  Fail,
}

#[derive(Clone, Copy)]
pub enum Op1 {
  Dec,
  Inc,
  Neg,
  Not,
}

// NB:
//
// we normalize some comparison operators
// - x != y => not (x == y)
// - x >= y => y <= x
// - x > y => y < x

#[derive(Clone, Copy)]
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

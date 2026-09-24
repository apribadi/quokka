//! untyped linear intermediate represenation

type Symbol = usize;

type Arity = usize;
type Error = usize;
type Field = usize;
type Index = usize;
type Label = usize;
type Local = usize;
type Value = usize;

#[derive(Clone, Copy)]
pub enum Inst {
  Label(Arity, Arity),
  GetArg(Index),
  SetOut(Index),
  Const(Symbol),
  ConstBool(bool),
  ConstI64(i64),
  DefLocal(Value),
  GetLocal(Local),
  SetLocal(Local, Value),
  GetField(Value, Field),
  SetField(Value, Field, Value),
  GetIndex(Value, Value),
  SetIndex(Value, Value, Value),
  Op1(Op1, Value),
  Op2(Op2, Value, Value),
  Call(Value),
  Cond(Value),
  Goto(Label),
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
// we normalize some comparisons
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

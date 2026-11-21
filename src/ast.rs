use crate::symbol::Symbol;

#[derive(Debug)]
pub enum Item {
  Fun(Box<Fun>),
}

#[derive(Debug)]
pub struct Fun {
  pub name: Option<Symbol>,
  pub args: Box<[Bind]>,
  pub body: Box<[Stmt]>,
}

#[derive(Debug)]
pub enum Stmt {
  ExprList(Box<[Expr]>),               // a, b
  Let(Box<[Bind]>, Box<[Expr]>),       // let x, y = a, b
  DefLocal(Option<Symbol>, Expr),      // let mutable x = a
  SetLocal(Symbol, Expr),              // x = 1
  SetField(Expr, Symbol, Expr),        // a.foo = 1
  SetIndex(Expr, Expr, Expr),          // a[b] = c
  SetLocalCompound(Symbol, Op2, Expr), // x += a
  Break(Box<[Expr]>),                  // break a, b
  While(Expr, Box<[Stmt]>),            // while (a) { ... }
  Continue,                            // continue
  Ret(Box<[Expr]>),                    // return a, b
}

#[derive(Debug)]
pub enum Expr {
  LiteralBool(bool),                             // true OR false
  LiteralI64(i64),                               // 3
  GetField(Box<(Expr, Symbol)>),                 // a.foo
  GetIndex(Box<(Expr, Expr)>),                   // a[b]
  If(Box<(Expr, Box<[Stmt]>)>),                  // if (a) { ... }
  IfElse(Box<(Expr, Box<[Stmt]>, Box<[Stmt]>)>), // if (a) { ... } else { ... }
  Op1(Box<(Op1, Expr)>),                         // - a
  Op2(Box<(Op2, Expr, Expr)>),                   // a + b
  ShortCircuitAnd(Box<(Expr, Expr)>),            // a && b
  ShortCircuitOr(Box<(Expr, Expr)>),             // a || b
  Variable(Symbol),                              // x
  Call(Box<(Expr, Box<[Expr]>)>),                // a(b, c)
  Loop(Box<[Stmt]>),                             // loop { ... }
  Fail,
}

// TODO: add optional type ascription

#[derive(Debug)]
pub struct Bind {
  pub name: Option<Symbol>,
}

#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum Op1 {
  BitComplement, // ^ a
  Neg, // - a
  Not, // ! a
}

#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum Op2 {
  Add,    // a + b
  BitAnd, // a & b
  BitOr,  // a | b
  BitXor, // a ^ b
  CmpEq,  // a == b
  CmpGe,  // a >= b
  CmpGt,  // a > b
  CmpLe,  // a <= b
  CmpLt,  // a < b
  CmpNe,  // a != b
  Div,    // a / b
  Mul,    // a * b
  Rem,    // a % b
  Shl,    // a << b
  Shr,    // a >> b
  Sub,    // a - b
}

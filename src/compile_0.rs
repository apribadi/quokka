//! lowering pass
//!
//! abstract syntax tree -> linearized bytecode + item references

// #![allow(warnings)]

use crate::prelude::*;
use crate::ast;
use crate::ast::Expr;
use crate::ast::Stmt;
use crate::ssa_0 as ssa;
use crate::ssa_0::Inst;
use crate::symbol::Symbol;

pub fn compile(item_list: &[ast::Item]) -> ssa::Module {
  let mut ctx = Ctx::new();

  for ast::Item::Fun(f) in item_list.iter() {
    let name = f.name;
    let pos = ctx.out.code.len();
    ctx.scope.enter();
    let _ = ctx.out.emit_label(f.args.len());
    for (i, s) in f.args.iter().enumerate() {
      let x = ctx.out.emit(Inst::Pop(i));
      if let Some(s) = s.name {
        ctx.scope.bind(s, Referent::Value(x))
      }
    }
    ctx.compile_block_tail(&f.body);
    ctx.scope.leave();
    let len = ctx.out.code.len() - pos;
    ctx.out.funs.push(ssa::FunDef { name, pos, len });
  }

  ssa::Module {
    funs: ctx.out.funs.into_boxed_slice(),
    code: ctx.out.code.into_boxed_slice(),
  }
}

type Value = usize;
type Local = usize;

#[derive(Clone, Copy)]
enum Referent {
  Local(Local),
  Value(Value),
}

#[derive(Clone, Copy)]
struct Point {
  instp: usize,
  index: usize,
  arity: Option<usize>,
}

const DUMMY: usize = usize::MAX;

#[derive(Clone, Copy)]
struct Label {
  instp: usize, // instruction pointer
  arity: usize,
}

enum LoopKind {
  TopLevel,
  NonTail { head: Label, break_count: usize },
  Tail { head: Label },
}

struct Ctx {
  stack: StackCtx,
  loops: LoopsCtx,
  scope: ScopeCtx,
  out: Out,
}

struct StackCtx {
  value: Vec<Value>,
  point: Vec<Point>,
  other: Vec<Point>,
}

struct LoopsCtx {
  kind: LoopKind,
  stack: Vec<LoopKind>,
  break_stack: Vec<Point>,
}

struct ScopeCtx {
  table: HashMap<Symbol, Referent>,
  stack: Vec<usize>,
  entry_count: usize,
  entry: Vec<(Symbol, Option<Referent>)>,
}

struct Out {
  block_label: usize,
  funs: Vec<ssa::FunDef>,
  code: Vec<ssa::Inst>,
}

trait IntoWhat: Sized {
  type What;

  fn yield_value(self, _: Value, _: &mut Ctx) -> Self::What;

  fn yield_tuple(self, _: usize, _: &mut Ctx) -> Self::What;

  fn yield_point_list(self, _: usize, _: &mut Ctx) -> Self::What;
}

struct IntoNil;

impl IntoWhat for IntoNil {
  type What = ();

  fn yield_value(self, value: Value, ctx: &mut Ctx) -> Self::What {
    // error, arity mismatch
    let _ = value;
    let _ = ctx.out.emit(Inst::Fail);
    let _ = ctx.out.emit_label(0);
  }

  fn yield_tuple(self, arity: usize, ctx: &mut Ctx) -> Self::What {
    if arity != 0 {
      // error, arity mismatch
      let _ = ctx.stack.value.pop_list(arity);
      let _ = ctx.out.emit(Inst::Fail);
      let _ = ctx.out.emit_label(0);
    }
  }

  fn yield_point_list(self, point_count: usize, ctx: &mut Ctx) -> Self::What {
    let a = ctx.out.emit_label(0);
    for p in ctx.stack.point.pop_list(point_count) {
      ctx.out.patch(p, a)
    }
  }
}

struct IntoValue;

impl IntoWhat for IntoValue {
  type What = Value;

  fn yield_value(self, value: Value, _: &mut Ctx) -> Self::What {
    value
  }

  fn yield_tuple(self, arity: usize, ctx: &mut Ctx) -> Self::What {
    if arity == 1 {
      ctx.stack.value.pop().unwrap()
    } else {
      // error, arity mismatch
      let _ = ctx.stack.value.pop_list(arity);
      let _ = ctx.out.emit(Inst::Fail);
      let _ = ctx.out.emit_label(1);
      ctx.out.emit(Inst::Pop(0))
    }
  }

  fn yield_point_list(self, point_count: usize, ctx: &mut Ctx) -> Self::What {
    let a = ctx.out.emit_label(1);
    for p in ctx.stack.point.pop_list(point_count) {
      ctx.out.patch(p, a)
    }
    ctx.out.emit(Inst::Pop(0))
  }
}

struct IntoValueList(usize);

impl IntoWhat for IntoValueList {
  type What = ();

  fn yield_value(self, value: Value, ctx: &mut Ctx) -> Self::What {
    if self.0 == 1 {
      ctx.stack.value.push(value);
    } else {
      // error, arity mismatch
      let _ = ctx.out.emit(Inst::Fail);
      let _ = ctx.out.emit_label(self.0);
      for i in 0 .. self.0 {
        ctx.stack.value.push(ctx.out.emit(Inst::Pop(i)));
      }
    }
  }

  fn yield_tuple(self, arity: usize, ctx: &mut Ctx) -> Self::What {
    if self.0 != arity {
      // error, arity mismatch
      let _ = ctx.stack.value.pop_list(arity);
      let _ = ctx.out.emit(Inst::Fail);
      let _ = ctx.out.emit_label(self.0);
      for i in 0 .. self.0 {
        ctx.stack.value.push(ctx.out.emit(Inst::Pop(i)));
      }
    }
  }

  fn yield_point_list(self, point_count: usize, ctx: &mut Ctx) -> Self::What {
    let a = ctx.out.emit_label(self.0);
    for p in ctx.stack.point.pop_list(point_count) {
      ctx.out.patch(p, a)
    }
    for i in 0 .. self.0 {
      ctx.stack.value.push(ctx.out.emit(Inst::Pop(i)));
    }
  }
}

struct IntoPointList;

impl IntoWhat for IntoPointList {
  type What = usize;

  fn yield_value(self, value: Value, ctx: &mut Ctx) -> Self::What {
    ctx.out.emit_puts([value]);
    ctx.stack.point.push(ctx.out.emit_jump(1));
    1
  }

  fn yield_tuple(self, arity: usize, ctx: &mut Ctx) -> Self::What {
    ctx.out.emit_puts(ctx.stack.value.pop_list(arity));
    ctx.stack.point.push(ctx.out.emit_jump(arity));
    1
  }

  fn yield_point_list(self, point_count: usize, ctx: &mut Ctx) -> Self::What {
    let _ = ctx;
    point_count
  }
}

struct IntoBackEdge(Label);

impl IntoWhat for IntoBackEdge {
  type What = ();

  fn yield_value(self, value: Value, ctx: &mut Ctx) -> Self::What {
    ctx.out.emit_puts([value]);
    let p = ctx.out.emit_jump(1);
    ctx.out.patch(p, self.0);
  }

  fn yield_tuple(self, arity: usize, ctx: &mut Ctx) -> Self::What {
    ctx.out.emit_puts(ctx.stack.value.pop_list(arity));
    let p = ctx.out.emit_jump(arity);
    ctx.out.patch(p, self.0);
  }

  fn yield_point_list(self, point_count: usize, ctx: &mut Ctx) -> Self::What {
    for p in ctx.stack.point.pop_list(point_count) {
      ctx.out.patch(p, self.0)
    }
  }
}

impl LoopsCtx {
  fn enter(&mut self, head: Label) {
    self.stack.push(replace(&mut self.kind, LoopKind::NonTail { head, break_count: 0 }));
  }

  fn leave(&mut self, points: &mut Vec<Point>) -> usize {
    match replace(&mut self.kind, self.stack.pop().unwrap()) {
    | LoopKind::NonTail { break_count, .. } => {
      for p in self.break_stack.pop_list(break_count) {
        points.push(p);
      }
      break_count
    }
    | _ =>
      unreachable!(),
    }
  }

  fn enter_tail(&mut self, head: Label) {
    self.stack.push(replace(&mut self.kind, LoopKind::Tail { head }));
  }

  fn leave_tail(&mut self) {
    match replace(&mut self.kind, self.stack.pop().unwrap()) {
    | LoopKind::Tail { .. } =>
      (),
    | _ =>
      unreachable!(),
    }
  }
}

impl ScopeCtx {
  fn enter(&mut self) {
    let n = replace(&mut self.entry_count, 0);
    self.stack.push(n);
  }

  fn leave(&mut self) {
    let n = replace(&mut self.entry_count, self.stack.pop().unwrap());
    for (s, x) in self.entry.pop_list(n) {
      if let Some(x) = x {
        self.table.insert(s, x);
      } else {
        self.table.remove(&s);
      }
    }
  }

  fn bind(&mut self, s: Symbol, x: Referent) {
    let y = self.table.insert(s, x);
    self.entry_count += 1;
    self.entry.push((s, y));
  }

  fn lookup(&self, s: Symbol) -> Option<&Referent> {
    self.table.get(&s)
  }
}

impl Out {
  fn emit(&mut self, inst: Inst) -> usize {
    self.code.emit(inst)
  }


  fn emit_jump(&mut self, arity: usize) -> Point {
    let i = self.emit(Inst::Jump(DUMMY));
    Point { instp: i, index: 0, arity: Some(arity) }
  }

  fn emit_cond(&mut self, value: usize) -> (Point, Point) {
    let i = self.emit(Inst::Cond(value, DUMMY, DUMMY));
    let p = Point { instp: i, index: 0, arity: Some(0) };
    let q = Point { instp: i, index: 1, arity: Some(0) };
    (p, q)
  }

  fn emit_call(&mut self, value: usize) -> Point {
    let i = self.emit(Inst::Call(value, DUMMY));
    Point { instp: i, index: 0, arity: None }
  }

  fn emit_label(&mut self, arity: usize) -> Label {
    let i = self.emit(Inst::Label(arity, 0));
    self.block_label = i;
    Label { instp: i, arity }
  }

  fn patch(&mut self, p: Point, a: Label) {
    if let Some(n) = p.arity && n != a.arity {
      // arity mismatch
      self.code[p.instp] = Inst::Fail;
    } else {
      match (&mut self.code[p.instp], p.index) {
      | (&mut Inst::Jump(ref mut point), 0) => {
        *point = a.instp;
      }
      | (&mut Inst::Cond(_, ref mut point, _), 0) => {
        *point = a.instp;
      }
      | (&mut Inst::Cond(_, _, ref mut point), 1) => {
        *point = a.instp;
      }
      | (&mut Inst::Call(_, ref mut point), 0) => {
        *point = a.instp;
      }
      | (&mut Inst::Fail, _) =>
        (),
      | _ =>
        unreachable!(),
      }
    }
  }

  fn emit_succ(&mut self, p: Point, n: usize) -> Label {
    let a = self.emit_label(n);
    self.patch(p, a);
    a
  }

  fn emit_puts(&mut self, xs: impl IntoIterator<Item = Value>) {
    let Inst::Label(_, n) = self.code[self.block_label] else { unreachable!() };
    let mut i = n;
    for x in xs {
      let _ = self.emit(Inst::Put(i, x));
      i += 1;
    }
    let Inst::Label(_, ref mut n) = self.code[self.block_label] else { unreachable!() };
    *n = i;
  }

  fn emit_cmp(&mut self, op: ast::Cmp, x: Value, y: Value) -> Value {
    let op =
      match op {
        ast::Cmp::Eq => ssa::Cmp::Eq,
        ast::Cmp::Le => ssa::Cmp::Le,
        ast::Cmp::Lt => ssa::Cmp::Lt,
        ast::Cmp::Ne => {
          let z = self.emit(Inst::Cmp(ssa::Cmp::Eq, x, y));
          return self.emit(Inst::Not(z));
        }
        ast::Cmp::Ge => {
          return self.emit(Inst::Cmp(ssa::Cmp::Le, y, x));
        }
        ast::Cmp::Gt => {
          return self.emit(Inst::Cmp(ssa::Cmp::Lt, y, x));
        }
      };
    self.emit(Inst::Cmp(op, x, y))
  }

  fn emit_op1(&mut self, op: ast::Op1, x: Value) -> Value {
    let op =
      match op {
        ast::Op1::BitComplement => ssa::Op1::BitComplement,
        ast::Op1::Neg => ssa::Op1::Neg,
      };
    self.emit(Inst::Op1(op, x))
  }

  fn emit_op2(&mut self, op: ast::Op2, x: Value, y: Value) -> Value {
    let op =
      match op {
      | ast::Op2::Add => ssa::Op2::Add,
      | ast::Op2::BitAnd => ssa::Op2::BitAnd,
      | ast::Op2::BitOr => ssa::Op2::BitOr,
      | ast::Op2::BitXor => ssa::Op2::BitXor,
      | ast::Op2::Div => ssa::Op2::Div,
      | ast::Op2::Mul => ssa::Op2::Mul,
      | ast::Op2::Rem => ssa::Op2::Rem,
      | ast::Op2::Shl => ssa::Op2::Shl,
      | ast::Op2::Shr => ssa::Op2::Shr,
      | ast::Op2::Sub => ssa::Op2::Sub,
      };
    self.emit(Inst::Op2(op, x, y))
  }
}

impl Ctx {
  fn new() -> Self {
    Self {
      stack: StackCtx {
        value: Vec::new(),
        point: Vec::new(),
        other: Vec::new(),
      },
      loops: LoopsCtx {
        kind: LoopKind::TopLevel,
        stack: Vec::new(),
        break_stack: Vec::new(),
      },
      scope: ScopeCtx {
        table: HashMap::new(),
        stack: Vec::new(),
        entry_count: 0,
        entry: Vec::new(),
      },
      out: Out {
        block_label: 0,
        funs: Vec::new(),
        code: Vec::new(),
      },
    }
  }

  fn compile_expr<I: IntoWhat>(&mut self, expr: &Expr, into: I) -> I::What {
    match expr {
    | &Expr::Call(ref e) => {
      let (ref f, ref args) = **e;
      for x in args.iter() {
        let x = self.compile_expr(x, IntoValue);
        self.stack.value.push(x);
      }
      let f = self.compile_expr(f, IntoValue); // NB: evaluated *after* args
      self.out.emit_puts(self.stack.value.pop_list(args.len()));
      let p = self.out.emit_call(f);
      self.stack.point.push(p);
      into.yield_point_list(1, self)
    }
    | &Expr::Fail => {
      let _ = self.out.emit(Inst::Fail);
      into.yield_point_list(0, self)
    }
    | &Expr::If(ref e) => {
      let (ref x, ref ys) = **e;
      let x = self.compile_expr(x, IntoValue);
      let p = self.out.emit_cond(x);
      self.stack.point.push(p.0); // false
      let _ = self.out.emit_succ(p.1, 0); // true
      let n = self.compile_block(ys, IntoPointList);
      into.yield_point_list(1 + n, self)
    }
    | &Expr::IfElse(ref e) => {
      let (ref x, ref ys, ref zs) = **e;
      let x = self.compile_expr(x, IntoValue);
      let p = self.out.emit_cond(x);
      let _ = self.out.emit_succ(p.0, 0); // false
      let m = self.compile_block(zs, IntoPointList);
      let _ = self.out.emit_succ(p.1, 0); // true
      let n = self.compile_block(ys, IntoPointList);
      into.yield_point_list(m + n, self)
    }
    | &Expr::GetField(ref e) => {
      let (ref x, s) = **e;
      let x = self.compile_expr(x, IntoValue);
      let y = self.out.emit(Inst::GetField(x, s));
      into.yield_value(y, self)
    }
    | &Expr::GetIndex(ref e) => {
      let (ref x, ref y) = **e;
      let y = self.compile_expr(y, IntoValue);
      let x = self.compile_expr(x, IntoValue);
      let z = self.out.emit(Inst::GetIndex(x, y));
      into.yield_value(z, self)
    }
    | &Expr::LiteralBool(x) => {
      let y = self.out.emit(Inst::ConstBool(x));
      into.yield_value(y, self)
    }
    | &Expr::LiteralI64(x) => {
      let y = self.out.emit(Inst::ConstI64(x));
      into.yield_value(y, self)
    }
    | &Expr::Loop(ref xs) => {
      let p = self.out.emit_jump(0);
      let a = self.out.emit_succ(p, 0);
      self.loops.enter(a);
      self.scope.enter();
      self.compile_block(xs, IntoBackEdge(a));
      self.scope.leave();
      let n = self.loops.leave(&mut self.stack.point);
      into.yield_point_list(n, self)
    }
    | &Expr::Cmp(ref e) => {
      let (op, ref x, ref y) = **e;
      let x = self.compile_expr(x, IntoValue);
      let y = self.compile_expr(y, IntoValue);
      let z = self.out.emit_cmp(op, x, y);
      into.yield_value(z, self)
    }
    | &Expr::Not(ref x) => {
      let x = self.compile_expr(x, IntoValue);
      let y = self.out.emit(Inst::Not(x));
      into.yield_value(y, self)
    }
    | &Expr::Op1(ref e) => {
      let (op, ref x) = **e;
      let x = self.compile_expr(x, IntoValue);
      let y = self.out.emit_op1(op, x);
      into.yield_value(y, self)
    }
    | &Expr::Op2(ref e) => {
      let (op, ref x, ref y) = **e;
      let x = self.compile_expr(x, IntoValue);
      let y = self.compile_expr(y, IntoValue);
      let z = self.out.emit_op2(op, x, y);
      into.yield_value(z, self)
    }
    | &Expr::ShortCircuitAnd(ref e) => {
      let (ref x, ref y) = **e;
      let x = self.compile_expr(x, IntoValue);
      let p = self.out.emit_cond(x);
      let _ = self.out.emit_succ(p.0, 0);
      let z = self.out.emit(Inst::ConstBool(false));
      self.out.emit_puts([z]);
      self.stack.point.push(self.out.emit_jump(1));
      let _ = self.out.emit_succ(p.1, 0);
      let n = self.compile_expr(y, IntoPointList);
      into.yield_point_list(1 + n, self)
    }
    | &Expr::ShortCircuitOr(ref e) => {
      let (ref x, ref y) = **e;
      let x = self.compile_expr(x, IntoValue);
      let p = self.out.emit_cond(x);
      let _ = self.out.emit_succ(p.0, 0);
      let n = self.compile_expr(y, IntoPointList);
      let _ = self.out.emit_succ(p.1, 0);
      let z = self.out.emit(Inst::ConstBool(true));
      self.out.emit_puts([z]);
      self.stack.point.push(self.out.emit_jump(1));
      into.yield_point_list(n + 1, self)
    }
    | &Expr::Variable(s) => {
      let x =
        match self.scope.lookup(s) {
          None =>
            self.out.emit(Inst::Const(s)),
          Some(&Referent::Local(v)) =>
            self.out.emit(Inst::GetLocal(v)),
          Some(&Referent::Value(x)) =>
            x,
        };
      into.yield_value(x, self)
    }
    }
  }

  fn compile_expr_tail(&mut self, expr: &Expr) {
    match expr {
      &Expr::Call(ref e) => {
        let (ref f, ref args) = **e;
        for x in args.iter() {
          let x = self.compile_expr(x, IntoValue);
          self.stack.value.push(x);
        }
        let f = self.compile_expr(f, IntoValue); // NB: evaluate *after* args
        self.out.emit_puts(self.stack.value.pop_list(args.len()));
        let _ = self.out.emit(Inst::TailCall(f));
      }
      &Expr::Fail => {
        let _ = self.out.emit(Inst::Fail);
      }
      &Expr::If(ref e) => {
        let (ref x, ref ys) = **e;
        let x = self.compile_expr(x, IntoValue);
        let p = self.out.emit_cond(x);
        let _ = self.out.emit_succ(p.0, 0); // false
        let _ = self.out.emit(Inst::Ret);
        let _ = self.out.emit_succ(p.1, 0);
        self.compile_block_tail(ys);
      }
      &Expr::IfElse(ref e) => {
        let (ref x, ref ys, ref zs) = **e;
        let x = self.compile_expr(x, IntoValue);
        let p = self.out.emit_cond(x);
        let _ = self.out.emit_succ(p.0, 0); // false
        self.compile_block_tail(zs);
        let _ = self.out.emit_succ(p.1, 0); // true
        self.compile_block_tail(ys);
      }
      &Expr::Loop(ref xs) => {
        let p = self.out.emit_jump(0);
        let a = self.out.emit_succ(p, 0);
        self.loops.enter_tail(a);
        self.scope.enter();
        self.compile_block(xs, IntoBackEdge(a));
        self.scope.leave();
        self.loops.leave_tail();
      }
      &Expr::ShortCircuitAnd(ref e) => {
        let (ref x, ref y) = **e;
        let x = self.compile_expr(x, IntoValue);
        let p = self.out.emit_cond(x);
        let _ = self.out.emit_succ(p.0, 0);
        let z = self.out.emit(Inst::ConstBool(false));
        self.out.emit_puts([z]);
        let _ = self.out.emit(Inst::Ret);
        let _ = self.out.emit_succ(p.1, 0);
        self.compile_expr_tail(y);
      }
      &Expr::ShortCircuitOr(ref e) => {
        let (ref x, ref y) = **e;
        let x = self.compile_expr(x, IntoValue);
        let p = self.out.emit_cond(x);
        let _ = self.out.emit_succ(p.0, 0);
        self.compile_expr_tail(y);
        let _ = self.out.emit_succ(p.1, 0);
        let z = self.out.emit(Inst::ConstBool(true));
        self.out.emit_puts([z]);
        let _ = self.out.emit(Inst::Ret);
      }
    | &Expr::LiteralBool(..)
    | &Expr::LiteralI64(..)
    | &Expr::GetField(..)
    | &Expr::GetIndex(..)
    | &Expr::Cmp(..)
    | &Expr::Not(..)
    | &Expr::Op1(..)
    | &Expr::Op2(..)
    | &Expr::Variable(..) => {
      let x = self.compile_expr(expr, IntoValue);
      self.out.emit_puts([x]);
      let _ = self.out.emit(Inst::Ret);
    }
    }
  }

  fn compile_stmt<I: IntoWhat>(&mut self, stmt: &Stmt, into: I) -> I::What {
    match stmt {
      &Stmt::ExprList(ref xs) => {
        self.compile_expr_list(xs, into)
      }
      &Stmt::Break(ref xs) => {
        match self.loops.kind {
          LoopKind::TopLevel => {
            // error, break is not inside loop
            let _ = self.out.emit(Inst::Fail);
          }
          LoopKind::NonTail { .. } => {
            let n = self.compile_expr_list(xs, IntoPointList);
            let &mut LoopKind::NonTail { ref mut break_count, .. } = &mut self.loops.kind else {
              unreachable!()
            };
            for p in self.stack.point.pop_list(n) {
              self.loops.break_stack.push(p);
              *break_count += 1;
            }
          }
          LoopKind::Tail { .. } => {
            self.compile_expr_list_tail(xs);
          }
        }
        into.yield_point_list(0, self)
      }
      &Stmt::Continue => {
        match self.loops.kind {
          LoopKind::TopLevel => {
            // error, break is not inside loop
            let _ = self.out.emit(Inst::Fail);
          }
          LoopKind::NonTail { head, .. } | LoopKind::Tail { head } => {
            let _ = self.out.emit(Inst::Jump(head.instp));
          }
        }
        into.yield_point_list(0, self)
      }
      &Stmt::Let(ref ss, ref xs) => {
        let n = ss.len();
        self.compile_expr_list(xs, IntoValueList(n));
        // NB: we bind from left to right, so later bindings shadow earlier
        // ones.
        for (s, x) in ss.iter().zip(self.stack.value.pop_list(n)) {
          if let Some(s) = s.name {
            self.scope.bind(s, Referent::Value(x));
          }
        }
        into.yield_tuple(0, self)
      }
      &Stmt::Ret(ref xs) => {
        self.compile_expr_list_tail(xs);
        into.yield_point_list(0, self)
      }
      &Stmt::SetLocal(s, ref x) => {
        let x = self.compile_expr(x, IntoValue);
        if let Some(&Referent::Local(v)) = self.scope.lookup(s) {
          let _ = self.out.emit(Inst::SetLocal(v, x));
          into.yield_tuple(0, self)
        } else {
          // error, symbol does not refer to local variable
          let _ = self.out.emit(Inst::Fail);
          into.yield_point_list(0, self)
        }
      }
      &Stmt::SetLocalCompound(s, op, ref x) => {
        let x = self.compile_expr(x, IntoValue);
        if let Some(&Referent::Local(v)) = self.scope.lookup(s) {
          let y = self.out.emit(Inst::GetLocal(v));
          let z = self.out.emit_op2(op, y, x);
          let _ = self.out.emit(Inst::SetLocal(v, z));
          into.yield_tuple(0, self)
        } else {
          // error, symbol does not refer to local variable
          let _ = self.out.emit(Inst::Fail);
          into.yield_point_list(0, self)
        }
      }
      &Stmt::SetField(ref x, s, ref y) => {
        let y = self.compile_expr(y, IntoValue);
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::SetField(x, s, y));
        into.yield_tuple(0, self)
      }
      &Stmt::SetIndex(ref x, ref y, ref z) => {
        let y = self.compile_expr(y, IntoValue);
        let z = self.compile_expr(z, IntoValue);
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::SetIndex(x, y, z));
        into.yield_tuple(0, self)
      }
      &Stmt::DefLocal(s, ref x) => {
        let x = self.compile_expr(x, IntoValue);
        let v = self.out.emit(Inst::DefLocal(x));
        if let Some(s) = s {
          self.scope.bind(s, Referent::Local(v));
        }
        into.yield_tuple(0, self)
      }
      &Stmt::While(ref x, ref ys) => {
        let p = self.out.emit_jump(0);
        let a = self.out.emit_succ(p, 0);
        let x = self.compile_expr(x, IntoValue);
        let q = self.out.emit_cond(x);
        self.stack.point.push(q.0);
        let _ = self.out.emit_succ(q.1, 0);
        self.loops.enter(a);
        self.scope.enter();
        self.compile_block(ys, IntoBackEdge(a));
        self.scope.leave();
        let n = self.loops.leave(&mut self.stack.point);
        into.yield_point_list(1 + n, self)
      }
    }
  }

  fn compile_stmt_tail(&mut self, stmt: &Stmt) {
    match stmt {
      &Stmt::ExprList(ref xs) => {
        self.compile_expr_list_tail(xs);
      }
      &Stmt::While(ref x, ref ys) => {
        let p = self.out.emit_jump(0);
        let a = self.out.emit_succ(p, 0);
        let x = self.compile_expr(x, IntoValue);
        let q = self.out.emit_cond(x);
        let _ = self.out.emit_succ(q.0, 0);
        let _ = self.out.emit(Inst::Ret);
        let _ = self.out.emit_succ(q.1, 0);
        self.loops.enter_tail(a);
        self.scope.enter();
        self.compile_block(ys, IntoBackEdge(a));
        self.scope.leave();
        self.loops.leave_tail();
      }
      &Stmt::Break(..)
      | &Stmt::Continue
      | &Stmt::Ret(..) => {
        let n = self.compile_stmt(stmt, IntoPointList);
        assert!(n == 0);
      }
      &Stmt::Let(..)
      | &Stmt::DefLocal(..)
      | &Stmt::SetLocal(..)
      | &Stmt::SetLocalCompound(..)
      | &Stmt::SetField(..)
      | &Stmt::SetIndex(..) => {
        self.compile_stmt(stmt, IntoValueList(0));
        let _ = self.out.emit(Inst::Ret);
      }
    }
  }

  fn compile_expr_list<I: IntoWhat>(&mut self, xs: &[Expr], into: I) -> I::What {
    if let [ref x] = *xs {
      self.compile_expr(x, into)
    } else {
      for x in xs.iter() {
        let x = self.compile_expr(x, IntoValue);
        self.stack.value.push(x);
      }
      into.yield_tuple(xs.len(), self)
    }
  }

  fn compile_expr_list_tail(&mut self, xs: &[Expr]) {
    if let [ref x] = *xs {
      self.compile_expr_tail(x);
    } else {
      for x in xs.iter() {
        let x = self.compile_expr(x, IntoValue);
        self.stack.value.push(x);
      }
      self.out.emit_puts(self.stack.value.pop_list(xs.len()));
      let _ = self.out.emit(Inst::Ret);
    }
  }

  fn compile_block<I: IntoWhat>(&mut self, stmt_list: &[Stmt], into: I) -> I::What {
    match stmt_list.split_last() {
      None => {
        into.yield_tuple(0, self)
      }
      Some((y, xs)) => {
        self.scope.enter();
        for x in xs {
          self.compile_stmt(x, IntoNil);
        }
        let y = self.compile_stmt(y, into);
        self.scope.leave();
        y
      }
    }
  }

  fn compile_block_tail(&mut self, stmt_list: &[Stmt]) {
    match stmt_list.split_last() {
      None => {
        let _ = self.out.emit(Inst::Ret);
      }
      Some((y, xs)) => {
        self.scope.enter();
        for x in xs {
          self.compile_stmt(x, IntoNil);
        }
        self.compile_stmt_tail(y);
        self.scope.leave();
      }
    }
  }
}

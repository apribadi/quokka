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
use crate::symbol::SymbolTable;
use std::mem::replace;

pub fn compile<'a>(item_list: &[ast::Item<'a>]) -> ssa::Module {
  let mut ctx = Ctx::new();

  for ast::Item::Fun(f) in item_list.iter() {
    let name = f.name.map(|s| ctx.symbol_table.intern(s));
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
    ctx.out.fun_defs.push(ssa::FunDef { name, pos, len });
  }

  ssa::Module {
    fun_defs: ctx.out.fun_defs.into_boxed_slice(),
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
  index: usize,
  arity: Option<usize>,
}

#[derive(Clone, Copy)]
struct Label {
  index: usize,
  arity: usize,
}

enum LoopKind {
  TopLevel,
  NonTail { head: Label, break_count: usize },
  Tail { head: Label },
}

struct Ctx<'a> {
  stack: StackCtx,
  loops: LoopsCtx,
  scope: ScopeCtx<'a>,
  symbol_table: SymbolTable,
  out: Out,
}

struct StackCtx {
  value: Vec<Value>,
  point: Vec<Point>,
}

struct LoopsCtx {
  kind: LoopKind,
  stack: Vec<LoopKind>,
  break_stack: Vec<Point>,
}

struct ScopeCtx<'a> {
  table: HashMap<&'a str, Referent>,
  stack: Vec<usize>,
  entry_count: usize,
  entry: Vec<(&'a str, Option<Referent>)>,
}

struct Out {
  block_label: usize,
  fun_defs: Vec<ssa::FunDef>,
  code: Vec<ssa::Inst>,
}

#[derive(Clone, Copy)]
enum What {
  Value(Value),
  PointList(usize),
  ValueList(usize),
}

impl What {
  fn into_nil<'a>(self, ctx: &mut Ctx<'a>) {
    match self {
      What::Value(_) => {
        // error, arity mismatch
        let _ = ctx.out.emit(Inst::Fail);
        let _ = ctx.out.emit_label(0);
      }
      What::PointList(point_count) => {
        let a = ctx.out.emit_label(0);
        ctx.patch_list(point_count, a);
      }
      What::ValueList(0) => {
      }
      What::ValueList(value_count) => {
        // error, arity mismatch
        let _ = ctx.stack.value.pop_list(value_count);
        let _ = ctx.out.emit(Inst::Fail);
        let _ = ctx.out.emit_label(0);
      }
    }
  }

  fn into_value<'a>(self, ctx: &mut Ctx<'a>) -> Value {
    match self {
      What::Value(value) => {
        value
      }
      What::PointList(point_count) => {
        let a = ctx.out.emit_label(1);
        ctx.patch_list(point_count, a);
        ctx.out.emit(Inst::Pop(0))
      }
      What::ValueList(1) => {
        ctx.stack.value.pop().unwrap()
      }
      What::ValueList(value_count) => {
        // error, arity mismatch
        let _ = ctx.stack.value.pop_list(value_count);
        let _ = ctx.out.emit(Inst::Fail);
        let _ = ctx.out.emit_label(1);
        ctx.out.emit(Inst::Pop(0))
      }
    }
  }

  fn into_value_list<'a>(self, n: usize, ctx: &mut Ctx<'a>) {
    match self {
      What::Value(value) if n == 1 => {
        ctx.stack.value.push(value);
      }
      What::Value(_) => {
        // error, arity mismatch
        let _ = ctx.out.emit(Inst::Fail);
        let _ = ctx.out.emit_label(n);
        for i in 0 .. n {
          ctx.stack.value.push(ctx.out.emit(Inst::Pop(i)));
        }
      }
      What::PointList(point_count) => {
        let a = ctx.out.emit_label(n);
        ctx.patch_list(point_count, a);
        for i in 0 .. n {
          ctx.stack.value.push(ctx.out.emit(Inst::Pop(i)));
        }
      }
      What::ValueList(value_count) if n == value_count => {
      }
      What::ValueList(value_count) => {
        // error, arity mismatch
        let _ = ctx.stack.value.pop_list(value_count);
        let _ = ctx.out.emit(Inst::Fail);
        let _ = ctx.out.emit_label(n);
        for i in 0 .. n {
          ctx.stack.value.push(ctx.out.emit(Inst::Pop(i)));
        }
      }
    }
  }

  fn into_point_list<'a>(self, ctx: &mut Ctx<'a>) -> usize {
    match self {
      What::Value(value) => {
        ctx.out.emit_put_list([value]);
        ctx.stack.point.push(ctx.out.emit_point(1));
        1
      }
      What::PointList(point_count) => {
        point_count
      }
      What::ValueList(value_count) => {
        ctx.out.emit_put_list(ctx.stack.value.pop_list(value_count));
        ctx.stack.point.push(ctx.out.emit_point(value_count));
        1
      }
    }
  }

  fn patch<'a>(self, a: Label, ctx: &mut Ctx<'a>) {
    let n = self.into_point_list(ctx);
    ctx.patch_list(n, a);
  }
}

trait WhatInto {
  type Output;

  fn into<'a>(self, _: What, _: &mut Ctx<'a>) -> Self::Output;
}

struct IntoNil;

impl WhatInto for IntoNil {
  type Output = ();

  fn into<'a>(self, what: What, ctx: &mut Ctx<'a>) -> Self::Output {
    what.into_nil(ctx)
  }
}

struct IntoValue;

impl WhatInto for IntoValue {
  type Output = Value;

  fn into<'a>(self, what: What, ctx: &mut Ctx<'a>) -> Self::Output {
    what.into_value(ctx)
  }
}

struct IntoValueList(usize);

impl WhatInto for IntoValueList {
  type Output = ();

  fn into<'a>(self, what: What, ctx: &mut Ctx<'a>) -> Self::Output {
    what.into_value_list(self.0, ctx)
  }
}

struct IntoPointList;

impl WhatInto for IntoPointList {
  type Output = usize;

  fn into<'a>(self, what: What, ctx: &mut Ctx<'a>) -> Self::Output {
    what.into_point_list(ctx)
  }
}

struct IntoBackEdge(Label);

impl WhatInto for IntoBackEdge {
  type Output = ();

  fn into<'a>(self, what: What, ctx: &mut Ctx<'a>) -> Self::Output {
    what.patch(self.0, ctx)
  }
}

impl LoopsCtx {
  fn enter(&mut self, head: Label) {
    self.stack.push(replace(&mut self.kind, LoopKind::NonTail { head, break_count: 0 }));
  }

  fn leave(&mut self, points: &mut Vec<Point>) -> usize {
    match replace(&mut self.kind, self.stack.pop().unwrap()) {
      LoopKind::NonTail { break_count, .. } => {
        for p in self.break_stack.pop_list(break_count) {
          points.push(p);
        }
        break_count
      }
      _ =>
        unreachable!(),
    }
  }

  fn enter_tail(&mut self, head: Label) {
    self.stack.push(replace(&mut self.kind, LoopKind::Tail { head }));
  }

  fn leave_tail(&mut self) {
    match replace(&mut self.kind, self.stack.pop().unwrap()) {
      LoopKind::Tail { .. } =>
        (),
      _ =>
        unreachable!(),
    }
  }
}

impl<'a> ScopeCtx<'a> {
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

  fn bind(&mut self, s: &'a str, x: Referent) {
    let y = self.table.insert(s, x);
    self.entry_count += 1;
    self.entry.push((s, y));
  }

  fn lookup(&self, s: &'a str) -> Option<&Referent> {
    self.table.get(&s)
  }
}

impl Out {
  fn emit(&mut self, inst: Inst) -> usize {
    let i = self.code.len();
    self.code.push(inst);
    i
  }

  fn emit_point(&mut self, arity: usize) -> Point {
    let i = self.emit(Inst::Goto(usize::MAX));
    Point { index: i, arity: Some(arity) }
  }

  fn emit_label(&mut self, arity: usize) -> Label {
    let i = self.emit(Inst::Label(arity, 0));
    self.block_label = i;
    Label { index: i, arity }
  }

  fn patch(&mut self, p: Point, a: Label) {
    debug_assert_matches!(self.code[p.index], Inst::Goto(usize::MAX));
    if let Some(n) = p.arity && n != a.arity {
      self.code[p.index] = Inst::Fail; // arity mismatch
    } else {
      self.code[p.index] = Inst::Goto(a.index);
    }
  }

  fn emit_label_after(&mut self, p: Point, n: usize) -> Label {
    let a = self.emit_label(n);
    self.patch(p, a);
    a
  }

  fn emit_put_list(&mut self, xs: impl IntoIterator<Item = Value>) {
    let Inst::Label(_, n) = self.code[self.block_label] else { unreachable!() };
    let mut i = n;
    for x in xs {
      let _ = self.emit(Inst::Put(i, x));
      i += 1;
    }
    let Inst::Label(_, ref mut n) = self.code[self.block_label] else { unreachable!() };
    *n = i;
  }

  fn emit_op1(&mut self, op: ast::Op1, x: Value) -> Value {
    let op =
      match op {
        ast::Op1::Neg => ssa::Op1::Neg,
        ast::Op1::Not => ssa::Op1::Not,
      };
    self.emit(Inst::Op1(op, x))
  }

  fn emit_op2(&mut self, op: ast::Op2, x: Value, y: Value) -> Value {
    let op =
      match op {
        ast::Op2::Add => ssa::Op2::Add,
        ast::Op2::BitAnd => ssa::Op2::BitAnd,
        ast::Op2::BitOr => ssa::Op2::BitOr,
        ast::Op2::BitXor => ssa::Op2::BitXor,
        ast::Op2::CmpEq => ssa::Op2::CmpEq,
        ast::Op2::CmpLe => ssa::Op2::CmpLe,
        ast::Op2::CmpLt => ssa::Op2::CmpLt,
        ast::Op2::Div => ssa::Op2::Div,
        ast::Op2::Mul => ssa::Op2::Mul,
        ast::Op2::Rem => ssa::Op2::Rem,
        ast::Op2::Shl => ssa::Op2::Shl,
        ast::Op2::Shr => ssa::Op2::Shr,
        ast::Op2::Sub => ssa::Op2::Sub,
        ast::Op2::CmpNe => {
          let z = self.emit(Inst::Op2(ssa::Op2::CmpEq, x, y));
          return self.emit(Inst::Op1(ssa::Op1::Not, z));
        }
        ast::Op2::CmpGe => {
          return self.emit(Inst::Op2(ssa::Op2::CmpLe, y, x));
        }
        ast::Op2::CmpGt => {
          return self.emit(Inst::Op2(ssa::Op2::CmpLt, y, x));
        }
      };
    self.emit(Inst::Op2(op, x, y))
  }
}

impl<'a> Ctx<'a> {
  fn new() -> Self {
    Self {
      stack: StackCtx {
        value: Vec::new(),
        point: Vec::new(),
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
      symbol_table: SymbolTable::new(),
      out: Out {
        block_label: 0,
        fun_defs: Vec::new(),
        code: Vec::new(),
      },
    }
  }

  fn patch_list(&mut self, n: usize, a: Label) {
    for p in self.stack.point.pop_list(n) {
      self.out.patch(p, a)
    }
  }

  fn compile_expr<W: WhatInto>(&mut self, expr: &Expr<'a>, into: W) -> W::Output {
    match *expr {
      Expr::Call(ref e) => {
        let (ref f, ref args) = **e;
        for x in args.iter() {
          let x = self.compile_expr(x, IntoValue);
          self.stack.value.push(x);
        }
        let f = self.compile_expr(f, IntoValue); // NB: evaluated *after* args
        self.out.emit_put_list(self.stack.value.pop_list(args.len()));
        let _ = self.out.emit(Inst::Call(f));
        let p = self.out.emit(Inst::Goto(usize::MAX));
        self.stack.point.push(Point { index: p, arity: None });
        into.into(What::PointList(1), self)
      }
      Expr::Fail => {
        let _ = self.out.emit(Inst::Fail);
        into.into(What::PointList(0), self)
      }
      Expr::If(ref e) => {
        let (ref x, ref ys) = **e;
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::Cond(x));
        self.stack.point.push(self.out.emit_point(0));
        let q = self.out.emit_point(0);
        let _ = self.out.emit_label_after(q, 0); // true
        let n = self.compile_block(ys, IntoPointList);
        into.into(What::PointList(1 + n), self)
      }
      Expr::IfElse(ref e) => {
        let (ref x, ref ys, ref zs) = **e;
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(0);
        let q = self.out.emit_point(0);
        let _ = self.out.emit_label_after(p, 0); // false
        let m = self.compile_block(zs, IntoPointList);
        let _ = self.out.emit_label_after(q, 0); // true
        let n = self.compile_block(ys, IntoPointList);
        into.into(What::PointList(m + n), self)
      }
      Expr::GetField(ref e) => {
        let (ref x, s) = **e;
        let s = self.symbol_table.intern(s);
        let x = self.compile_expr(x, IntoValue);
        let y = self.out.emit(Inst::GetField(x, s));
        into.into(What::Value(y), self)
      }
      Expr::GetIndex(ref e) => {
        let (ref x, ref y) = **e;
        let y = self.compile_expr(y, IntoValue);
        let x = self.compile_expr(x, IntoValue);
        let z = self.out.emit(Inst::GetIndex(x, y));
        into.into(What::Value(z), self)
      }
      Expr::LiteralBool(x) => {
        let y = self.out.emit(Inst::ConstBool(x));
        into.into(What::Value(y), self)
      }
      Expr::LiteralI64(x) => {
        let y = self.out.emit(Inst::ConstI64(x));
        into.into(What::Value(y), self)
      }
      Expr::Loop(ref xs) => {
        let p = self.out.emit_point(0);
        let a = self.out.emit_label_after(p, 0);
        self.loops.enter(a);
        self.scope.enter();
        self.compile_block(xs, IntoBackEdge(a));
        self.scope.leave();
        let n = self.loops.leave(&mut self.stack.point);
        into.into(What::PointList(n), self)
      }
      Expr::Op1(ref e) => {
        let (op, ref x) = **e;
        let x = self.compile_expr(x, IntoValue);
        let y = self.out.emit_op1(op, x);
        into.into(What::Value(y), self)
      }
      Expr::Op2(ref e) => {
        let (op, ref x, ref y) = **e;
        let x = self.compile_expr(x, IntoValue);
        let y = self.compile_expr(y, IntoValue);
        let z = self.out.emit_op2(op, x, y);
        into.into(What::Value(z), self)
      }
      Expr::Select(ref e) => {
        let (ref x, ref y, ref z) = **e;
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(0);
        let q = self.out.emit_point(0);
        let _ = self.out.emit_label_after(p, 0); // false
        let m = self.compile_expr(z, IntoPointList);
        let _ = self.out.emit_label_after(q, 0); // true
        let n = self.compile_expr(y, IntoPointList);
        into.into(What::PointList(m + n), self)
      }
      Expr::ShortCircuitAnd(ref e) => {
        let (ref x, ref y) = **e;
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(0);
        let q = self.out.emit_point(0);
        let _ = self.out.emit_label_after(p, 0);
        let z = self.out.emit(Inst::ConstBool(false));
        self.out.emit_put_list([z]);
        self.stack.point.push(self.out.emit_point(1));
        let _ = self.out.emit_label_after(q, 0);
        let n = self.compile_expr(y, IntoPointList);
        into.into(What::PointList(1 + n), self)
      }
      Expr::ShortCircuitOr(ref e) => {
        let (ref x, ref y) = **e;
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(0);
        let q = self.out.emit_point(0);
        let _ = self.out.emit_label_after(p, 0);
        let n = self.compile_expr(y, IntoPointList);
        let _ = self.out.emit_label_after(q, 0);
        let z = self.out.emit(Inst::ConstBool(true));
        self.out.emit_put_list([z]);
        self.stack.point.push(self.out.emit_point(1));
        into.into(What::PointList(n + 1), self)
      }
      Expr::Variable(s) => {
        let x =
          match self.scope.lookup(s) {
            None =>
              self.out.emit(Inst::Const(self.symbol_table.intern(s))),
            Some(&Referent::Local(v)) =>
              self.out.emit(Inst::GetLocal(v)),
            Some(&Referent::Value(x)) =>
              x,
          };
        into.into(What::Value(x), self)
      }
    }
  }

  fn compile_expr_tail(&mut self, expr: &Expr<'a>) {
    match *expr {
      Expr::Call(ref e) => {
        let (ref f, ref args) = **e;
        for x in args.iter() {
          let x = self.compile_expr(x, IntoValue);
          self.stack.value.push(x);
        }
        let f = self.compile_expr(f, IntoValue); // NB: evaluate *after* args
        self.out.emit_put_list(self.stack.value.pop_list(args.len()));
        let _ = self.out.emit(Inst::TailCall(f));
      }
      Expr::Fail => {
        let _ = self.out.emit(Inst::Fail);
      }
      Expr::If(ref e) => {
        let (ref x, ref ys) = **e;
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(0);
        let q = self.out.emit_point(0);
        let _ = self.out.emit_label_after(p, 0); // false
        let _ = self.out.emit(Inst::Ret);
        let _ = self.out.emit_label_after(q, 0);
        self.compile_block_tail(ys);
      }
      Expr::IfElse(ref e) => {
        let (ref x, ref ys, ref zs) = **e;
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(0);
        let q = self.out.emit_point(0);
        let _ = self.out.emit_label_after(p, 0); // false
        self.compile_block_tail(zs);
        let _ = self.out.emit_label_after(q, 0); // true
        self.compile_block_tail(ys);
      }
      Expr::Loop(ref xs) => {
        let p = self.out.emit_point(0);
        let a = self.out.emit_label_after(p, 0);
        self.loops.enter_tail(a);
        self.scope.enter();
        self.compile_block(xs, IntoBackEdge(a));
        self.scope.leave();
        self.loops.leave_tail();
      }
      Expr::Select(ref e) => {
        let (ref x, ref y, ref z) = **e;
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(0);
        let q = self.out.emit_point(0);
        let _ = self.out.emit_label_after(p, 0); // false
        self.compile_expr_tail(z);
        let _ = self.out.emit_label_after(q, 0); // true
        self.compile_expr_tail(y);
      }
      Expr::ShortCircuitAnd(ref e) => {
        let (ref x, ref y) = **e;
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(0);
        let q = self.out.emit_point(0);
        let _ = self.out.emit_label_after(p, 0);
        let z = self.out.emit(Inst::ConstBool(false));
        self.out.emit_put_list([z]);
        let _ = self.out.emit(Inst::Ret);
        let _ = self.out.emit_label_after(q, 0);
        self.compile_expr_tail(y);
      }
      Expr::ShortCircuitOr(ref e) => {
        let (ref x, ref y) = **e;
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(0);
        let q = self.out.emit_point(0);
        let _ = self.out.emit_label_after(p, 0);
        self.compile_expr_tail(y);
        let _ = self.out.emit_label_after(q, 0);
        let z = self.out.emit(Inst::ConstBool(true));
        self.out.emit_put_list([z]);
        let _ = self.out.emit(Inst::Ret);
      }
      Expr::LiteralBool(..)
      | Expr::LiteralI64(..)
      | Expr::GetField(..)
      | Expr::GetIndex(..)
      | Expr::Op1(..)
      | Expr::Op2(..)
      | Expr::Variable(..) => {
        let x = self.compile_expr(expr, IntoValue);
        self.out.emit_put_list([x]);
        let _ = self.out.emit(Inst::Ret);
      }
    }
  }

  fn compile_stmt<W: WhatInto>(&mut self, stmt: &Stmt<'a>, into: W) -> W::Output {
    match *stmt {
      Stmt::ExprList(ref xs) => {
        self.compile_expr_list(xs, into)
      }
      Stmt::Break(ref xs) => {
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
        into.into(What::PointList(0), self)
      }
      Stmt::Continue => {
        match self.loops.kind {
          LoopKind::TopLevel => {
            // error, break is not inside loop
            let _ = self.out.emit(Inst::Fail);
          }
          LoopKind::NonTail { head, .. } | LoopKind::Tail { head } => {
            let _ = self.out.emit(Inst::Goto(head.index));
          }
        }
        into.into(What::PointList(0), self)
      }
      Stmt::Let(ref ss, ref xs) => {
        let n = ss.len();
        self.compile_expr_list(xs, IntoValueList(n));
        // NB: we bind from left to right, so later bindings shadow earlier
        // ones.
        for (s, x) in ss.iter().zip(self.stack.value.pop_list(n)) {
          if let Some(s) = s.name {
            self.scope.bind(s, Referent::Value(x));
          }
        }
        into.into(What::ValueList(0), self)
      }
      Stmt::Ret(ref xs) => {
        self.compile_expr_list_tail(xs);
        into.into(What::PointList(0), self)
      }
      Stmt::SetLocal(s, ref x) => {
        let x = self.compile_expr(x, IntoValue);
        if let Some(&Referent::Local(v)) = self.scope.lookup(s) {
          let _ = self.out.emit(Inst::SetLocal(v, x));
          into.into(What::ValueList(0), self)
        } else {
          // error, symbol does not refer to local variable
          let _ = self.out.emit(Inst::Fail);
          into.into(What::PointList(0), self)
        }
      }
      Stmt::SetLocalCompound(s, op, ref x) => {
        let x = self.compile_expr(x, IntoValue);
        if let Some(&Referent::Local(v)) = self.scope.lookup(s) {
          let y = self.out.emit(Inst::GetLocal(v));
          let z = self.out.emit_op2(op, y, x);
          let _ = self.out.emit(Inst::SetLocal(v, z));
          into.into(What::ValueList(0), self)
        } else {
          // error, symbol does not refer to local variable
          let _ = self.out.emit(Inst::Fail);
          into.into(What::PointList(0), self)
        }
      }
      Stmt::SetField(ref x, s, ref y) => {
        let s = self.symbol_table.intern(s);
        let y = self.compile_expr(y, IntoValue);
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::SetField(x, s, y));
        into.into(What::ValueList(0), self)
      }
      Stmt::SetIndex(ref x, ref y, ref z) => {
        let y = self.compile_expr(y, IntoValue);
        let z = self.compile_expr(z, IntoValue);
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::SetIndex(x, y, z));
        into.into(What::ValueList(0), self)
      }
      Stmt::DefLocal(s, ref x) => {
        let x = self.compile_expr(x, IntoValue);
        let v = self.out.emit(Inst::DefLocal(x));
        if let Some(s) = s {
          self.scope.bind(s, Referent::Local(v));
        }
        into.into(What::ValueList(0), self)
      }
      Stmt::While(ref x, ref ys) => {
        let p = self.out.emit_point(0);
        let a = self.out.emit_label_after(p, 0);
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::Cond(x));
        self.stack.point.push(self.out.emit_point(0));
        let q = self.out.emit_point(0);
        let _ = self.out.emit_label_after(q, 0);
        self.loops.enter(a);
        self.scope.enter();
        self.compile_block(ys, IntoBackEdge(a));
        self.scope.leave();
        let n = self.loops.leave(&mut self.stack.point);
        into.into(What::PointList(1 + n), self)
      }
    }
  }

  fn compile_stmt_tail(&mut self, stmt: &Stmt<'a>) {
    match *stmt {
      Stmt::ExprList(ref xs) => {
        self.compile_expr_list_tail(xs);
      }
      Stmt::While(ref x, ref ys) => {
        let p = self.out.emit_point(0);
        let a = self.out.emit_label_after(p, 0);
        let x = self.compile_expr(x, IntoValue);
        let _ = self.out.emit(Inst::Cond(x));
        let q = self.out.emit_point(0);
        let r = self.out.emit_point(0);
        let _ = self.out.emit_label_after(q, 0);
        let _ = self.out.emit(Inst::Ret);
        let _ = self.out.emit_label_after(r, 0);
        self.loops.enter_tail(a);
        self.scope.enter();
        self.compile_block(ys, IntoBackEdge(a));
        self.scope.leave();
        self.loops.leave_tail();
      }
      Stmt::Break(..)
      | Stmt::Continue
      | Stmt::Ret(..) => {
        let n = self.compile_stmt(stmt, IntoPointList);
        assert!(n == 0);
      }
      Stmt::Let(..)
      | Stmt::DefLocal(..)
      | Stmt::SetLocal(..)
      | Stmt::SetLocalCompound(..)
      | Stmt::SetField(..)
      | Stmt::SetIndex(..) => {
        self.compile_stmt(stmt, IntoValueList(0));
        let _ = self.out.emit(Inst::Ret);
      }
    }
  }

  fn compile_expr_list<W: WhatInto>(&mut self, xs: &[Expr<'a>], into: W) -> W::Output {
    if let [ref x] = *xs {
      self.compile_expr(x, into)
    } else {
      for x in xs.iter() {
        let x = self.compile_expr(x, IntoValue);
        self.stack.value.push(x);
      }
      into.into(What::ValueList(xs.len()), self)
    }
  }

  fn compile_expr_list_tail(&mut self, xs: &[Expr<'a>]) {
    if let [ref x] = *xs {
      self.compile_expr_tail(x);
    } else {
      for x in xs.iter() {
        let x = self.compile_expr(x, IntoValue);
        self.stack.value.push(x);
      }
      self.out.emit_put_list(self.stack.value.pop_list(xs.len()));
      let _ = self.out.emit(Inst::Ret);
    }
  }

  fn compile_block<W: WhatInto>(&mut self, stmt_list: &[Stmt<'a>], into: W) -> W::Output {
    match stmt_list.split_last() {
      None => {
        into.into(What::ValueList(0), self)
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

  fn compile_block_tail(&mut self, stmt_list: &[Stmt<'a>]) {
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

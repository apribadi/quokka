//! lowering pass
//!
//! abstract syntax tree -> linearized bytecode + item references

#![allow(warnings)]

use crate::prelude::*;
use crate::ast;
use crate::ast::Expr;
use crate::ast::Stmt;
use crate::ssa_0 as ssa;
use crate::ssa_0::Inst;
use crate::symbol::Symbol;
use crate::symbol::SymbolTable;
use std::mem::replace;

type Value = usize;
type Local = usize;

#[derive(Clone, Copy)]
enum What {
  NumPoints(usize),
  NumValues(usize),
}

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
  code: Vec<ssa::Inst>,
}

impl What {
  const NEVER: Self = What::NumPoints(0);

  const NIL: Self = What::NumValues(0);

  fn into_nil(self, ctx: &mut Ctx) {
    match self {
      What::NumPoints(num_points) => {
        let a = ctx.out.emit_label(0);
        ctx.patch_list(num_points, a);
      }
      What::NumValues(0) => {
      }
      What::NumValues(num_values) => {
        // error, arity mismatch
        let _ = ctx.stack.value.pop_list(num_values);
        let _ = ctx.out.emit(Inst::Fail);
        let _ = ctx.out.emit_label(0);
      }
    }
  }

  fn into_value(self, ctx: &mut Ctx) -> Value {
    match self {
      What::NumPoints(num_points) => {
        let a = ctx.out.emit_label(1);
        ctx.patch_list(num_points, a);
        ctx.out.emit(Inst::GetArg(0))
      }
      What::NumValues(1) => {
        ctx.stack.value.pop().unwrap()
      }
      What::NumValues(num_values) => {
        // error, arity mismatch
        let _ = ctx.stack.value.pop_list(num_values);
        let _ = ctx.out.emit(Inst::Fail);
        let _ = ctx.out.emit_label(1);
        ctx.out.emit(Inst::GetArg(0))
      }
    }
  }

  fn into_value_list(self, arity: usize, ctx: &mut Ctx) {
    match self {
      What::NumPoints(num_points) => {
        let a = ctx.out.emit_label(arity);
        ctx.patch_list(num_points, a);
        for i in 0 .. arity {
          ctx.stack.value.push(ctx.out.emit(Inst::GetArg(i)));
        }
      }
      What::NumValues(num_values) if arity == num_values => {
      }
      What::NumValues(num_values) => {
        // error, arity mismatch
        let _ = ctx.stack.value.pop_list(num_values);
        let _ = ctx.out.emit(Inst::Fail);
        let _ = ctx.out.emit_label(arity);
        for i in 0 .. arity {
          ctx.stack.value.push(ctx.out.emit(Inst::GetArg(i)));
        }
      }
    }
  }

  fn into_point_list(self, ctx: &mut Ctx) -> usize {
    match self {
      What::NumPoints(num_points) => {
        num_points
      }
      What::NumValues(num_values) => {
        for (i, x) in ctx.stack.value.pop_list(num_values).enumerate() {
          let _ = ctx.out.emit(Inst::SetOut(i, x));
        }
        // TODO: exity
        ctx.stack.point.push(ctx.out.emit_point(Some(num_values)));
        1
      }
    }
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

  fn emit_point(&mut self, arity: Option<usize>) -> Point {
    let i = self.emit(Inst::Jump(usize::MAX));
    Point { index: i, arity }
  }

  fn emit_label(&mut self, arity: usize) -> Label {
    let i = self.emit(Inst::Label(arity, 0));
    self.block_label = i;
    Label { index: i, arity }
  }

  fn patch(&mut self, p: Point, a: Label) {
    if let Some(n) = p.arity && n != a.arity {
      self.code[p.index] = Inst::Fail; // arity mismatch
    } else {
      self.code[p.index] = Inst::Jump(a.index);
    }
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
        code: Vec::new(),
      },
    }
  }

  fn patch_list(&mut self, n: usize, a: Label) {
    for p in self.stack.point.pop_list(n) {
      self.out.patch(p, a)
    }
  }

  fn compile_expr(&mut self, expr: &Expr<'a>) -> What {
    match *expr {
      Expr::Call(ref e) => {
        let (ref f, ref args) = **e;
        let n = args.len();
        for x in args.iter() {
          let x = self.compile_expr(x).into_value(self);
          self.stack.value.push(x);
        }
        let f = self.compile_expr(f).into_value(self); // NB: evaluate *after* args
        for (i, x) in self.stack.value.pop_list(n).enumerate() {
          let _ = self.out.emit(Inst::SetOut(i, x));
        }
        let _ = self.out.emit(Inst::Call(f));
        self.stack.point.push(self.out.emit_point(None));
        What::NumPoints(1)
      }
      Expr::Fail => {
        let _ = self.out.emit(Inst::Fail);
        What::NEVER
      }
      Expr::If(ref e) => {
        let (ref x, ref ys) = **e;
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(Some(0));
        let q = self.out.emit_point(Some(0));
        self.stack.point.push(p);
        let a = self.out.emit_label(0);
        self.out.patch(q, a);
        let n = self.compile_block(ys).into_point_list(self);
        What::NumPoints(1 + n)
      }
      Expr::IfElse(ref e) => {
        let (ref x, ref ys, ref zs) = **e;
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(Some(0));
        let q = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0); // false
        self.out.patch(p, a);
        let m = self.compile_block(zs).into_point_list(self);
        let b = self.out.emit_label(0); // true
        self.out.patch(q, b);
        let n = self.compile_block(ys).into_point_list(self);
        What::NumPoints(m + n)
      }
      Expr::GetField(ref e) => {
        let (ref x, s) = **e;
        let s = self.symbol_table.intern(s);
        let x = self.compile_expr(x).into_value(self);
        self.stack.value.push(self.out.emit(Inst::GetField(x, s)));
        What::NumValues(1)
      }
      Expr::GetIndex(ref e) => {
        let (ref x, ref y) = **e;
        let y = self.compile_expr(y).into_value(self);
        let x = self.compile_expr(x).into_value(self);
        self.stack.value.push(self.out.emit(Inst::GetIndex(x, y)));
        What::NumValues(1)
      }
      Expr::LiteralBool(x) => {
        self.stack.value.push(self.out.emit(Inst::ConstBool(x)));
        What::NumValues(1)
      }
      Expr::LiteralI64(x) => {
        self.stack.value.push(self.out.emit(Inst::ConstI64(x)));
        What::NumValues(1)
      }
      Expr::Loop(ref xs) => {
        let p = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0);
        self.out.patch(p, a);
        self.loops.enter(a);
        let m = self.compile_block(xs).into_point_list(self);
        self.patch_list(m, a);
        let n = self.loops.leave(&mut self.stack.point);
        What::NumPoints(n)
      }
      Expr::Op1(ref e) => {
        let (op, ref x) = **e;
        let x = self.compile_expr(x).into_value(self);
        self.compile_op1(op, x)
      }
      Expr::Op2(ref e) => {
        let (op, ref x, ref y) = **e;
        let x = self.compile_expr(x).into_value(self);
        let y = self.compile_expr(y).into_value(self);
        self.compile_op2(op, x, y)
      }
      Expr::Select(ref e) => {
        let (ref x, ref y, ref z) = **e;
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(Some(0));
        let q = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0); // false
        self.out.patch(p, a);
        let m = self.compile_expr(z).into_point_list(self);
        let b = self.out.emit_label(0); // true
        self.out.patch(q, b);
        let n = self.compile_expr(y).into_point_list(self);
        What::NumPoints(m + n)
      }
      Expr::ShortCircuitAnd(ref e) => {
        let (ref x, ref y) = **e;
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(Some(0));
        let q = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0);
        self.out.patch(p, a);
        let z = self.out.emit(Inst::ConstBool(false));
        let _ = self.out.emit(Inst::SetOut(0, z));
        let r = self.out.emit_point(Some(1));
        self.stack.point.push(r);
        let b = self.out.emit_label(0);
        self.out.patch(q, b);
        let n = self.compile_expr(y).into_point_list(self);
        What::NumPoints(1 + n)
      }
      Expr::ShortCircuitOr(ref e) => {
        let (ref x, ref y) = **e;
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(Some(0));
        let q = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0);
        self.out.patch(p, a);
        let n = self.compile_expr(y).into_point_list(self);
        let b = self.out.emit_label(0);
        self.out.patch(q, b);
        let z = self.out.emit(Inst::ConstBool(true));
        let _ = self.out.emit(Inst::SetOut(0, z));
        let r = self.out.emit_point(Some(1));
        self.stack.point.push(r);
        What::NumPoints(n + 1)
      }
      Expr::Variable(s) => {
        match self.scope.lookup(s) {
          None => {
            self.stack.value.push(self.out.emit(Inst::Const(self.symbol_table.intern(s))));
          }
          Some(&Referent::Local(v)) => {
            self.stack.value.push(self.out.emit(Inst::GetLocal(v)))
          }
          Some(&Referent::Value(x)) => {
            self.stack.value.push(x)
          }
        }
        What::NumValues(1)
      }
    }
  }

  fn compile_expr_tail(&mut self, expr: &Expr<'a>) {
    match *expr {
      Expr::Call(ref e) => {
        let (ref f, ref args) = **e;
        let n = args.len();
        for x in args.iter() {
          let x = self.compile_expr(x).into_value(self);
          self.stack.value.push(x);
        }
        let f = self.compile_expr(f).into_value(self); // NB: evaluate *after* args
        for (i, x) in self.stack.value.pop_list(n).enumerate() {
          let _ = self.out.emit(Inst::SetOut(i, x));
        }
        let _ = self.out.emit(Inst::TailCall(f));
      }
      Expr::Fail => {
        let _ = self.out.emit(Inst::Fail);
      }
      Expr::If(ref e) => {
        let (ref x, ref ys) = **e;
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(Some(0));
        let q = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0); // false
        self.out.patch(p, a);
        let _ = self.out.emit(Inst::Ret);
        let b = self.out.emit_label(0);
        self.out.patch(q, b);
        self.compile_block_tail(ys);
      }
      Expr::IfElse(ref e) => {
        let (ref x, ref ys, ref zs) = **e;
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(Some(0));
        let q = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0); // false
        self.out.patch(p, a);
        self.compile_block_tail(zs);
        let b = self.out.emit_label(0); // true
        self.out.patch(q, b);
        self.compile_block_tail(ys);
      }
      Expr::Loop(ref xs) => {
        let p = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0);
        self.out.patch(p, a);
        self.loops.enter_tail(a);
        let n = self.compile_block(xs).into_point_list(self);
        self.patch_list(n, a);
        self.loops.leave_tail();
      }
      Expr::Select(ref e) => {
        let (ref x, ref y, ref z) = **e;
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(Some(0));
        let q = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0); // false
        self.out.patch(p, a);
        self.compile_expr_tail(z);
        let b = self.out.emit_label(0); // true
        self.out.patch(q, b);
        self.compile_expr_tail(y);
      }
      Expr::ShortCircuitAnd(ref e) => {
        let (ref x, ref y) = **e;
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(Some(0));
        let q = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0);
        self.out.patch(p, a);
        let z = self.out.emit(Inst::ConstBool(false));
        let _ = self.out.emit(Inst::SetOut(0, z));
        let _ = self.out.emit(Inst::Ret);
        let b = self.out.emit_label(0);
        self.out.patch(q, b);
        self.compile_expr_tail(y);
      }
      Expr::ShortCircuitOr(ref e) => {
        let (ref x, ref y) = **e;
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::Cond(x));
        let p = self.out.emit_point(Some(0));
        let q = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0);
        self.out.patch(p, a);
        self.compile_expr_tail(y);
        let b = self.out.emit_label(0);
        self.out.patch(q, b);
        let z = self.out.emit(Inst::ConstBool(true));
        let _ = self.out.emit(Inst::SetOut(0, z));
        let _ = self.out.emit(Inst::Ret);
      }
      Expr::LiteralBool(..)
      | Expr::LiteralI64(..)
      | Expr::GetField(..)
      | Expr::GetIndex(..)
      | Expr::Op1(..)
      | Expr::Op2(..)
      | Expr::Variable(..) => {
        let What::NumValues(1) = self.compile_expr(expr) else { unreachable!() };
        let x = self.stack.value.pop().unwrap();
        let _ = self.out.emit(Inst::SetOut(0, x));
        let _ = self.out.emit(Inst::Ret);
      }
    }
  }

  fn compile_stmt(&mut self, stmt: &Stmt<'a>) -> What {
    match *stmt {
      Stmt::ExprList(ref xs) => {
        self.compile_expr_list(xs)
      }
      Stmt::Break(ref xs) => {
        match self.loops.kind {
          LoopKind::TopLevel => {
            // error, break is not inside loop
            let _ = self.out.emit(Inst::Fail);
          }
          LoopKind::NonTail { .. } => {
            let n = self.compile_expr_list(xs).into_point_list(self);
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
        What::NEVER
      }
      Stmt::Continue => {
        match self.loops.kind {
          LoopKind::TopLevel => {
            // error, break is not inside loop
            let _ = self.out.emit(Inst::Fail);
          }
          LoopKind::NonTail { head, .. } | LoopKind::Tail { head } => {
            // NB: currently, all loop headers have arity zero
            let _ = self.out.emit(Inst::Jump(head.index));
          }
        }
        What::NEVER
      }
      Stmt::Let(ref ss, ref xs) => {
        let n = ss.len();
        // NB: we do the bindings from left to right, so later bindings shadow
        // earlier ones.
        self.compile_expr_list(xs).into_value_list(n, self);
        for (s, x) in ss.iter().zip(self.stack.value.pop_list(n)) {
          if let Some(s) = s.name {
            self.scope.bind(s, Referent::Value(x));
          }
        }
        What::NIL
      }
      Stmt::Ret(ref xs) => {
        self.compile_expr_list_tail(xs);
        What::NEVER
      }
      Stmt::SetLocal(s, ref x) => {
        let x = self.compile_expr(x).into_value(self);
        if let Some(&Referent::Local(v)) = self.scope.lookup(s) {
          let _ = self.out.emit(Inst::SetLocal(v, x));
          What::NIL
        } else {
          // error, symbol does not refer to local variable
          let _ = self.out.emit(Inst::Fail);
          What::NEVER
        }
      }
      Stmt::SetLocalCompound(s, op, ref x) => {
        let x = self.compile_expr(x).into_value(self);
        if let Some(&Referent::Local(v)) = self.scope.lookup(s) {
          let y = self.out.emit(Inst::GetLocal(v));
          let z = self.compile_op2(op, y, x).into_value(self);
          let _ = self.out.emit(Inst::SetLocal(v, z));
          What::NIL
        } else {
          // error, symbol does not refer to local variable
          let _ = self.out.emit(Inst::Fail);
          What::NEVER
        }
      }
      Stmt::SetField(ref x, s, ref y) => {
        let s = self.symbol_table.intern(s);
        let y = self.compile_expr(y).into_value(self);
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::SetField(x, s, y));
        What::NIL
      }
      Stmt::SetIndex(ref x, ref y, ref z) => {
        let y = self.compile_expr(y).into_value(self);
        let z = self.compile_expr(z).into_value(self);
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::SetIndex(x, y, z));
        What::NIL
      }
      Stmt::DefLocal(s, ref x) => {
        let x = self.compile_expr(x).into_value(self);
        let v = self.out.emit(Inst::DefLocal(x));
        if let Some(s) = s {
          self.scope.bind(s, Referent::Local(v));
        }
        What::NIL
      }
      Stmt::While(ref x, ref ys) => {
        let p = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0);
        self.out.patch(p, a);
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::Cond(x));
        let q = self.out.emit_point(Some(0));
        let r = self.out.emit_point(Some(0));
        self.stack.point.push(q);
        let b = self.out.emit_label(0);
        self.out.patch(r, b);
        self.loops.enter(a);
        let m = self.compile_block(ys).into_point_list(self);
        self.patch_list(m, a);
        let n = self.loops.leave(&mut self.stack.point);
        What::NumPoints(1 + n)
      }
    }
  }

  fn compile_stmt_tail(&mut self, stmt: &Stmt<'a>) {
    match *stmt {
      Stmt::ExprList(ref xs) => {
        self.compile_expr_list_tail(xs);
      }
      Stmt::While(ref x, ref ys) => {
        let p = self.out.emit_point(Some(0));
        let a = self.out.emit_label(0);
        self.out.patch(p, a);
        let x = self.compile_expr(x).into_value(self);
        let _ = self.out.emit(Inst::Cond(x));
        let q = self.out.emit_point(Some(0));
        let r = self.out.emit_point(Some(0));
        let b = self.out.emit_label(0);
        self.out.patch(q, b);
        let _ = self.out.emit(Inst::Ret);
        let c = self.out.emit_label(0);
        self.out.patch(r, c);
        self.loops.enter_tail(a);
        let m = self.compile_block(ys).into_point_list(self);
        self.patch_list(m, a);
        self.loops.leave_tail();
      }
      Stmt::Break(..)
      | Stmt::Continue
      | Stmt::Ret(..) => {
        let What::NumPoints(0) = self.compile_stmt(stmt) else { unreachable!() };
      }
      Stmt::Let(..)
      | Stmt::SetLocal(..)
      | Stmt::SetLocalCompound(..)
      | Stmt::SetField(..)
      | Stmt::SetIndex(..)
      | Stmt::DefLocal(..) => {
        let What::NumValues(0) = self.compile_stmt(stmt) else { unreachable!() };
        let _ = self.out.emit(Inst::Ret);
      }
    }
  }

  fn compile_expr_list(&mut self, expr_list: &[Expr<'a>]) -> What {
    if let [ref x] = *expr_list {
      self.compile_expr(x)
    } else {
      let n = expr_list.len();
      for x in expr_list.iter() {
        let x = self.compile_expr(x).into_value(self);
        self.stack.value.push(x);
      }
      What::NumValues(n)
    }
  }

  fn compile_expr_list_tail(&mut self, expr_list: &[Expr<'a>]) {
    if let [ref x] = *expr_list {
      self.compile_expr_tail(x);
    } else {
      let n = expr_list.len();
      for x in expr_list.iter() {
        let x = self.compile_expr(x).into_value(self);
        self.stack.value.push(x);
      }
      for (i, x) in self.stack.value.pop_list(n).enumerate() {
        let _ = self.out.emit(Inst::SetOut(i, x));
      }
      let _ = self.out.emit(Inst::Ret);
    }
  }

  fn compile_block(&mut self, stmt_list: &[Stmt<'a>]) -> What {
    match stmt_list.split_last() {
      None => {
        What::NIL
      }
      Some((y, xs)) => {
        self.scope.enter();
        for x in xs {
          self.compile_stmt(x).into_nil(self);
        }
        let y = self.compile_stmt(y);
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
          self.compile_stmt(x).into_nil(self);
        }
        self.compile_stmt_tail(y);
        self.scope.leave();
      }
    }
  }

  fn compile_op1(&mut self, op: ast::Op1, x: Value) -> What {
    let op =
      match op {
        ast::Op1::Neg => ssa::Op1::Neg,
        ast::Op1::Not => ssa::Op1::Not,
      };
    let y = self.out.emit(Inst::Op1(op, x));
    self.stack.value.push(y);
    What::NumValues(1)
  }

  fn compile_op2(&mut self, op: ast::Op2, x: Value, y: Value) -> What {
    let z = 'value: {
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
            let z = self.out.emit(Inst::Op2(ssa::Op2::CmpEq, x, y));
            let z = self.out.emit(Inst::Op1(ssa::Op1::Not, z));
            break 'value z;
          }
          ast::Op2::CmpGe => {
            let z = self.out.emit(Inst::Op2(ssa::Op2::CmpLe, y, x));
            break 'value z;
          }
          ast::Op2::CmpGt => {
            let z = self.out.emit(Inst::Op2(ssa::Op2::CmpLt, y, x));
            break 'value z;
          }
        };
      self.out.emit(Inst::Op2(op, x, y))
    };
    self.stack.value.push(z);
    What::NumValues(1)
  }
}

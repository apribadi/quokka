//! lowering pass
//!
//! abstract syntax tree -> linearized bytecode + item references

#![allow(warnings)]

use crate::prelude::*;
use crate::ast;
use crate::ssa_0 as ssa;
use crate::ssa_0::Inst;
use std::mem::replace;

type Value = usize;
type Local = usize;
type Symbol<'a> = &'a str;

enum Effect {
  Points(usize),
  Values(usize),
}

enum Referent {
  Local(Local),
  Value(Value),
}

struct Point {
  index: usize,
  arity: Option<usize>,
}

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
  values: Vec<Value>,
  points: Vec<Point>,
  block: BlockCtx,
  loops: LoopsCtx,
  scope: ScopeCtx<'a>,
  out: OutCtx,
}

struct BlockCtx {
  label: usize,
  exity: usize,
}

struct LoopsCtx {
  kind: LoopKind,
  stack: Vec<LoopKind>,
  break_stack: Vec<Point>,
}

struct ScopeCtx<'a> {
  table: HashMap<Symbol<'a>, Referent>,
  stack: Vec<usize>,
  entry_count: usize,
  entry_stack: Vec<(Symbol<'a>, Option<Referent>)>,
}

struct OutCtx {
  code: Vec<ssa::Inst>,
}

impl BlockCtx {
  fn new() -> Self {
    Self {
      label: 0,
      exity: 0,
    }
  }
}

impl LoopsCtx {
  fn new() -> Self {
    Self {
      kind: LoopKind::TopLevel,
      stack: Vec::new(),
      break_stack: Vec::new(),
    }
  }

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
  fn new() -> Self {
    Self {
      table: HashMap::new(),
      stack: Vec::new(),
      entry_count: 0,
      entry_stack: Vec::new(),
    }
  }

  fn enter(&mut self) {
    let n = replace(&mut self.entry_count, 0);
    self.stack.push(n);
  }

  fn leave(&mut self) {
    let n = replace(&mut self.entry_count, self.stack.pop().unwrap());
    for (s, x) in self.entry_stack.pop_list(n) {
      if let Some(x) = x {
        self.table.insert(s, x);
      } else {
        self.table.remove(&s);
      }
    }
  }

  fn bind(&mut self, s: Symbol<'a>, x: Referent) {
    let y = self.table.insert(s, x);
    self.entry_count += 1;
    self.entry_stack.push((s, y));
  }

  fn lookup(&self, s: Symbol<'a>) -> Option<&Referent> {
    self.table.get(&s)
  }
}

impl OutCtx {
  fn new() -> Self {
    Self {
      code: Vec::new(),
    }
  }

  fn emit(&mut self, inst: Inst) -> usize {
    let i = self.code.len();
    self.code.push(inst);
    i
  }

  fn emit_point(&mut self, arity: Option<usize>) -> Point {
    let i = self.emit(Inst::Jump(usize::MAX));
    Point { index: i, arity }
  }

  fn emit_label(&mut self, arity: usize, block: &mut BlockCtx) -> Label {
    let i = self.emit(Inst::Label(arity, usize::MAX));
    block.label = i;
    block.exity = 0;
    Label { index: i, arity }
  }

  fn patch(&mut self, a: Point, b: Label) {
    if let Some(n) = a.arity && n != b.arity {
      self.code[a.index] = Inst::Fail; // arity mismatch
    } else {
      self.code[a.index] = Inst::Jump(b.index);
    }
  }
}

impl<'a> Ctx<'a> {
  fn new() -> Self {
    Self {
      values: Vec::new(),
      points: Vec::new(),
      block: BlockCtx::new(),
      loops: LoopsCtx::new(),
      scope: ScopeCtx::new(),
      out: OutCtx::new(),
    }
  }
}

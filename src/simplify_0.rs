//!
//!
//!

// #![allow(warnings)]

use crate::prelude::*;
use crate::ssa_0::Inst;
use crate::ssa_0::FunDef;
use crate::ssa_0::Module;
use crate::ssa_0::iter_block_insts;

/*
enum Mult<A> {
  None,
  Many,
  Uniq(A),
}

struct Info {
  prev: Mult<usize>,
  next: Mult<usize>,
}
*/

fn mark(i: usize, a: usize, todo: &mut Vec<usize>, seen: &mut HashMap<usize, usize>) {
  let _ = i;
  match seen.entry(a) {
    Vacant(e) => {
      e.insert(1);
      todo.push(a);
    }
    Occupied(e) => {
      let e = e.into_mut();
      *e += 1;
    }
  }
}

pub fn simplify(module: &Module) -> Module {
  let Module { funs, code } = module;
  let mut remap = vec![usize::MAX; code.len()];
  let mut todo = Vec::new();
  let mut seen = HashMap::new();
  let mut o_code = Vec::new();
  let mut o_funs = Vec::new();
  for f in funs.iter() {
    seen.insert(f.pos, 0);
    todo.push(f.pos);
    while let Some(i) = todo.pop() {
      for (_, inst) in iter_block_insts(&code, i) {
        match inst {
          &Inst::Jump(a) => {
            mark(i, a, &mut todo, &mut seen);
          }
          &Inst::Cond(_, a, b) => {
            mark(i, a, &mut todo, &mut seen);
            mark(i, b, &mut todo, &mut seen);
          }
          &Inst::Call(_, a) => {
            mark(i, a, &mut todo, &mut seen);
          }
          _ => {
          }
        }
      }
    }
    let mut seen_: Vec<_> = seen.keys().cloned().collect();
    seen_.sort();
    for &i in seen_.iter() {
      for (i, &inst) in iter_block_insts(&code, i) {
        remap[i] = o_code.emit(inst);
      }
    }
    seen.clear();
    let o_f_pos = remap[f.pos];
    let o_f_end = o_code.len();
    let o_f_len = o_f_end - o_f_pos;
    for i in o_f_pos .. o_f_end {
      o_code[i] = o_code[i].remap(|i| remap[i]);
    }
    o_funs.push(
      FunDef {
        name: f.name,
        pos: o_f_pos,
        len: o_f_len,
      }
    );
  }
  Module {
    funs: o_funs.into_boxed_slice(),
    code: o_code.into_boxed_slice(),
  }
}

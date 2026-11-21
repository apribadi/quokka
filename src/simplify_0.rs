//!
//!
//!

// #![allow(warnings)]

use crate::prelude::*;
use crate::ssa_0::Inst;
use crate::ssa_0::FunDef;
use crate::ssa_0::Module;
use crate::ssa_0::bb_inst_iter;

pub fn simplify(module: &Module) -> Module {
  let Module { funs, code } = module;
  let mut remap = vec![usize::MAX; code.len()];
  let mut todo = Vec::new();
  let mut seen = HashMap::new();
  let mut o_code = Vec::new();
  let mut o_funs = Vec::new();
  for f in funs.iter() {
    seen.insert(f.pos, 1);
    todo.push(f.pos);
    while let Some(i) = todo.pop() {
      for (_, inst) in bb_inst_iter(&code, i) {
        if let Inst::Goto(a) = *inst {
          match seen.entry(a) {
            Vacant(e) => { e.insert(1); todo.push(a); }
            Occupied(e) => { *e.into_mut() += 1; }
          }
        }
      }
    }
    let mut seen_: Vec<_> = seen.keys().cloned().collect();
    seen_.sort();
    for &i in seen_.iter() {
      for (i, &inst) in bb_inst_iter(&code, i) {
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

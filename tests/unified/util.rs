/*
pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let module = quokka::parser::parse(source.as_bytes());
  write!(out, "{:#?}\n", module);
}

pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let module = quokka::parser::parse(source.as_bytes());
  write!(out, "{:#?}\n", module);
}
*/

pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let module = quokka::parser::parse(source.as_bytes());
  let module = quokka::compile_0::compile(&module);
  for f in module.fun_defs.iter() {
    match f.name {
      None => { write!(out, "=== fun _ ===\n"); }
      Some(s) => { write!(out, "=== fun {} ===\n", s); }
    }
    for i in f.pos .. f.pos + f.len {
      let inst = module.code[i];
      write!(out, "%{} {}\n", i, inst);
    }
  }
}
    /*
    match inst {
      | quokka::ssa_0::Inst::Fail
      | quokka::ssa_0::Inst::SetOut(..)
      | quokka::ssa_0::Inst::Goto(..)
      | quokka::ssa_0::Inst::Cond(..)
      | quokka::ssa_0::Inst::Ret
      | quokka::ssa_0::Inst::Call(..)
      | quokka::ssa_0::Inst::TailCall(..)
      | quokka::ssa_0::Inst::SetField(..)
      | quokka::ssa_0::Inst::SetIndex(..)
      | quokka::ssa_0::Inst::SetLocal(..) =>
        write!(out, "%{} {}\n", i, inst).unwrap(),
      | quokka::ssa_0::Inst::Get(..)
      | quokka::ssa_0::Inst::Const(..)
      | quokka::ssa_0::Inst::ConstBool(..)
      | quokka::ssa_0::Inst::ConstInt(..)
      | quokka::ssa_0::Inst::Field(..)
      | quokka::ssa_0::Inst::Index(..)
      | quokka::ssa_0::Inst::GetLocal(..)
      | quokka::ssa_0::Inst::Op1(..)
      | quokka::ssa_0::Inst::Op2(..) => {
        let x = lilac::typeid::TypeId(i);
        write!(out, "%{} {} : {}\n", i, inst, solver.resolve_value_type(x).unwrap()).unwrap();
      }
      | quokka::ssa_0::Inst::Local(..) => {
        let x = lilac::typeid::TypeId(i);
        write!(out, "%{} {} : Local {}\n", i, inst, solver.resolve_value_type(x).unwrap()).unwrap();
      }
      | quokka::ssa_0::Inst::Label(_) => {
        let x = solver.resolve_tuple_type(lilac::typeid::TypeId(i)).unwrap();
        write!(out, "%{} {} : {}\n", i, inst, x).unwrap();
      }
    }
    */

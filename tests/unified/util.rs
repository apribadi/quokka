pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let module = quokka::parser::parse(source.as_bytes());
  write!(out, "{:#?}\n", module);
}

/*
pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let mut store = oxcart::Store::new();

  let module = lilac::parse::parse(source.as_bytes(), store.arena()).0;
  let module = lilac::make_iru::compile(&module);

  let (environment, solver) = lilac::typecheck::typecheck(&module);

  for f in module.decl.iter() {
    write!(out, "=== fun {} : {} ===\n", f.name, environment[f.name]).unwrap();

    for i in f.pos .. f.pos + f.len {
      let inst = module.code[i];
      match inst {
        | lilac::iru::Inst::GotoStaticError
        | lilac::iru::Inst::Put(..)
        | lilac::iru::Inst::Goto(..)
        | lilac::iru::Inst::Cond(..)
        | lilac::iru::Inst::Ret
        | lilac::iru::Inst::Call(..)
        | lilac::iru::Inst::TailCall(..)
        | lilac::iru::Inst::SetField(..)
        | lilac::iru::Inst::SetIndex(..)
        | lilac::iru::Inst::SetLocal(..) =>
          write!(out, "%{} {}\n", i, inst).unwrap(),
        | lilac::iru::Inst::Get(..)
        | lilac::iru::Inst::Const(..)
        | lilac::iru::Inst::ConstBool(..)
        | lilac::iru::Inst::ConstInt(..)
        | lilac::iru::Inst::Field(..)
        | lilac::iru::Inst::Index(..)
        | lilac::iru::Inst::GetLocal(..)
        | lilac::iru::Inst::Op1(..)
        | lilac::iru::Inst::Op2(..) => {
          let x = lilac::typeid::TypeId(i);
          write!(out, "%{} {} : {}\n", i, inst, solver.resolve_value_type(x).unwrap()).unwrap();
        }
        | lilac::iru::Inst::Local(..) => {
          let x = lilac::typeid::TypeId(i);
          write!(out, "%{} {} : Local {}\n", i, inst, solver.resolve_value_type(x).unwrap()).unwrap();
        }
        | lilac::iru::Inst::Label(_) => {
          let x = solver.resolve_tuple_type(lilac::typeid::TypeId(i)).unwrap();
          write!(out, "%{} {} : {}\n", i, inst, x).unwrap();
        }
      }
    }
  }
}
*/

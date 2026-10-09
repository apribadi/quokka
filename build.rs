fn main() {
  lalrpop::process_src().unwrap();
  /*
  lalrpop::Configuration::new()
    .generate_in_source_tree()
    .process().unwrap();
  */
}

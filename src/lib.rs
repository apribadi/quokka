pub mod ast;
pub mod lexer;
pub mod parser;
pub mod prelude;
pub mod ssa_0;
pub mod token;
pub mod compile_0;
pub mod symbol;
// pub mod lower;

// syntax ideas - use something in { :, @, ` } to denote symbol/label ?
//
//   fun foo(...) -> .foo int:t, .bar int:t { ... }
//
//   fun foo(...) -> case @ok .foo int:t, .bar int:t | @error .baz int:t { ... }
//
//   fun foo(...) -> case .foo int:t, .bar int:t | .baz int:t { ... }
//
//   fun foo(...) -> int:t, int:t { ... }
//
//   fun foo(...) -> ! { ... }
//
//   fun foo(...) -> case ! { ... }
//
//   fun foo(...) -> case { ... }
//
//   fun foo(...) -> case | { ... }
//
//   fun foo(...) -> case @ok | @error { ... }
//
//   fun foo(...) -> case @continue | @break { ... }
//
//   fun foo(...) -> case || { ... }
//
//   fun foo(...) -> case int:t { ... }
//
//   fun foo(...) -> case int:t | int:t { ... }
//
//   fun foo(...) -> case @0 int:t | @1 int:t { ... }
//
//   fun foo(...) -> case @ok int:t | @error int:t { ... }
//
//   let x, y = z ...
//
//   let .foo ~ x, .bar ~ y = z ...
//
//   case z ... {
//      @ok x => { x + 1 }
//      @error => { 2 }
//   }
//
//   case z ... {
//      @ok .x, .y => { x + y }
//      @error .z => { z + 1 }
//   }
//
//   return @1
//
//   return @0 x, y
//
//   return @ok
//
//   return @error x, y
//
//   let ! = f(...)
//
//   case f(...) { }
//
//   case f(...) { ! }
//
//   case f(...) { x, y => { ... } => { ... } }
//
//   case f(...) { x, y => { ... } x => { ... } }
//
//   case f(...) { @0 x, y => { ... } @1 x => { ... } }
//
//   case f(...) { @ok x, y => { ... } @x => { ... } }
//
//   case f(...) {
//     => { ... }
//     => { ... }
//   }
//
//   case f(...) {
//     x, y => { ... }
//     x => { ... }
//   }
//
//   case f(...) {
//     @0 x, y => { ... }
//     @1 x => { ... }
//   }
//
//   case f(...) {
//     @none => { ... }
//     @some x => { ... }
//   }
//
//   with-cont {
//     ...
//     goto @foo 1, 2
//     ...
//     goto @bar
//   } {
//      @foo x, y => { ... }
//      @bar => { ... }
//   }
//
//   foo(2, @a = 1)
//
//   fun foo(x int:t, y int:t) -> int:t { ...  }
//
//   fun foo(x int:t, .y int:t) -> int:t { ... }
//
//   fun foo(x int:t, .y ~ z int:t) -> int:t { ... }
//
//   fun foo(int:t, .y int:t) -> int:t
//
//   let x = construct(.a = 1, .b = 2)
//
//   let x = construct(.a, .b)
//
//   let x = construct(.a ~ 1, .b ~ 2)
//
//   let .a, .b = x ...
//
//   let .a int, .b int = x ...
//
//   let .a ~ x, .b ~ y = x ...
//
//   let .a ~ x int, .b ~ y int = x ...
//
//   return .a ~ 2, .b ~ 3
//
//   return .a, .b
//
//   case f(...) {
//     .a, .b => { ... }
//     .a => { ... }
//   }
//
//   case f(...) {
//     @0 .a, .b => { ... }
//     @1 .a => { ... }
//   }
//
//   case f(...) {
//     @0 .a int, .b int => { ... }
//     @1 .a int => { ... }
//   }
//
//   case f(...) {
//     @0 .a ~ x, .b ~ y => { ... }
//     @1 .a ~ x=> { ... }
//   }
//
//   case f(...) {
//     @0 .a ~ x int, .b ~ y int => { ... }
//     @1 .a ~ x int => { ... }
//   }
//
//   @foo loop {
//     ...
//     loop {
//       break @foo
//     }
//     ...
//     loop {
//       continue @foo
//     }
//   }
//
//   let x = case f(...) { @ok y => { y } @error => { return @error } }
//
//   let x = match f(...) { Ok(y) => y, Err(()) => { return Err(()); } }
//
//   let x = f(...)?

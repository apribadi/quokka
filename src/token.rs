//! token tag
//!
//! NB: single-byte tokens are their own variant tag

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Token {
  Not        = 0x21, // !
  Hash       = 0x23, // #
  Dollar     = 0x24, // $
  Rem        = 0x25, // %
  BitAnd     = 0x26, // &
  Apostrophe = 0x27, // '
  LParen     = 0x28, // (
  RParen     = 0x29, // )
  Mul        = 0x2a, // *
  Add        = 0x2b, // +
  Comma      = 0x2c, // ,
  Hyphen     = 0x2d, // -
  Dot        = 0x2e, // .
  Div        = 0x2f, // /
  Colon      = 0x3a, // :
  CmpLt      = 0x3c, // <
  Equal      = 0x3d, // =
  CmpGt      = 0x3e, // >
  Query      = 0x3f, // ?
  At         = 0x40, // @
  LBracket   = 0x5b, // [
  RBracket   = 0x5d, // ]
  BitXor     = 0x5e, // ^
  Underscore = 0x5f, // _
  Backtick   = 0x60, // `
  LBrace     = 0x7b, // {
  BitOr      = 0x7c, // |
  RBrace     = 0x7d, // }
  Tilde      = 0x7e, // ~
  Eof        = 0xa0,
  Error,
  AddEqual,          // +=
  CmpEq,             // ==
  CmpGe,             // >=
  CmpLe,             // <=
  CmpNe,             // !=
  DotDotDot,         // ...
  Shl,               // <<
  ShortCircuitAnd,   // &&
  ShortCircuitOr,    // ||
  Shr,               // >>
  SubEqual,          // -=
  Break,
  Continue,
  Do,
  Elif,
  Else,
  False,
  For,
  Fun,
  If,
  Let,
  Loop,
  Mutable,
  Return,
  True,
  While,
  Symbol,
  Int,
  Quote,
}

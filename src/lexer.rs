use logos::Logos;
use crate::token::Token;

pub struct Lexer<'a>(logos::Lexer<'a, Token>);


impl<'a> Lexer<'a> {
  pub fn new(source: &'a [u8]) -> Self {
    Self(Token::lexer(source))
  }
}

impl<'a> Iterator for Lexer<'a> {
  type Item = Result<(usize, Token, usize), ()>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.0.next() {
      None => None,
      Some(Err(())) => Some(Err(())),
      Some(Ok(token)) => {
        let span = self.0.span();
        Some(Ok((span.start, token, span.end)))
      }
    }
  }
}

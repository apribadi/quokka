use crate::prelude::*;
use crate::ast::Binding;
use crate::ast::Expr;
use crate::ast::Fun;
use crate::ast::Item;
use crate::ast::Op1;
use crate::ast::Op2;
use crate::ast::PrePost;
use crate::ast::IncDec;
use crate::ast::Stmt;
use crate::lexer::scan;
use crate::token::Token;

/*
pub fn parse(source: &[u8]) -> Box<[Item<'a>]> {
  let mut t = T::new(source);
  t.parse_item_list();
  t.items.into_boxed_slice()
}
*/

struct Parser<'a, 'b> {
  source: &'b [(Token, bool, &'a [u8])],
  token_index: usize,
  token: Token,
  token_is_attached: bool,
  token_span: &'a [u8],
  items: Vec<Item<'a>>,
  bindings: Vec<Binding<'a>>,
  exprs: Vec<Expr<'a>>,
  stmts: Vec<Stmt<'a>>,
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum Priority {
  Any,
  Select,
  LogOr,
  LogAnd,
  Cmp,
  BitOr,
  BitXor,
  BitAnd,
  Shift,
  Add,
  Mul,
  Prefix,
}

fn next<'a, 'b>(source: &'b [(Token, bool, &'a [u8])]) -> ((Token, bool, &'a [u8]), &'b [(Token, bool, &'a [u8])]) {
  match source.split_first() {
    None => ((Token::Eof, false, &[]), &[]),
    Some((&(x, y, z), source)) => ((x, y, z), source),
  }
}

impl<'a, 'b> Parser<'a, 'b> {
  fn new(source: &'b [(Token, bool, &'a [u8])]) -> Self {
    let ((token, token_is_attached, token_span), source) = next(source);
    Self {
      source,
      token_index: 0,
      token,
      token_is_attached,
      token_span,
      items: Vec::new(),
      bindings: Vec::new(),
      exprs: Vec::new(),
      stmts: Vec::new(),
    }
  }

  fn next(&mut self) {
    let ((token, token_is_attached, token_span), source) = next(self.source);
    self.source = source;
    self.token_index += 1;
    self.token = token;
    self.token_is_attached = token_is_attached;
    self.token_span = token_span;
  }

  fn token_index(&self) -> usize {
    self.token_index
  }

  fn token(&self) -> Token {
    self.token
  }

  fn token_is_attached(&self) -> bool {
    self.token_is_attached
  }

  fn token_span(&self) -> &'b [u8] {
    self.token_span
  }

  fn expect(&mut self, token: Token) {
    if self.token() != token {
      self.on_error_missing_expected_token(token);
    } else {
      self.next();
    }
  }

  fn expect_symbol(&mut self) -> &'b [u8] {
    if self.token() != Token::Symbol {
      self.on_error_missing_expected_token(Token::Symbol);
      // TODO: Option::None?
      b"!!!"
    } else {
      let symbol = self.token_span();
      self.next();
      symbol
    }
  }

  /*

  fn parse_item_list(&mut self) {
    loop {
      match self.token() {
        Token::Eof => {
          break;
        }
        Token::Fun => {
          self.next();
          let name = self.expect_symbol();
          self.expect(Token::LParen);
          let m = self.parse_binding_list(Token::RParen);
          self.expect(Token::RParen);
          let n = self.parse_block();
          self.on_fun(name, m, n);
        }
        _ => {
          // TODO: error?
          break;
        }
      }
    }
  }

  fn parse_binding(&mut self) {
    match self.token() {
      Token::Symbol => {
        let s = self.token_span();
        self.on_binding(Some(s));
        self.next();
      }
      Token::Underscore => {
        self.on_binding(None);
        self.next();
      }
      _ => {
        self.on_error_missing_expected_token(Token::Symbol);
        self.on_binding(None);
      }
    }
  }

  fn parse_binding_list(&mut self, stop: Token) -> usize {
    let mut n_bindings = 0;
    if self.token() != stop {
      loop {
        self.parse_binding();
        n_bindings += 1;
        if self.token() != Token::Comma { break; }
        self.next();
      }
    }
    return n_bindings;
  }

  fn parse_expr(&mut self) {
    self.parse_expr_prec(P::Any);
  }

  fn parse_expr_list(&mut self, stop: Token) -> usize {
    let mut n_exprs = 0;
    if self.token() != stop {
      loop {
        self.parse_expr();
        n_exprs += 1;
        if self.token() != Token::Comma { break; }
        self.next();
      }
    }
    return n_exprs;
  }

  fn parse_expr_prec(&mut self, p: P) {
    let _: bool = self.parse_prec(p, false);
  }

  // returns `true` if we parsed a statement

  fn parse_prec(&mut self, p: P, is_stmt: bool) -> bool {
    match self.token() {
      Token::LParen => {
        self.next();
        self.parse_expr();
        self.expect(Token::RParen);
      }
      Token::True => {
        self.next();
        self.on_literal_bool(true);
      }
      Token::False => {
        self.next();
        self.on_literal_bool(false);
      }
      Token::Number => {
        let value = self.token_span();
        self.next();
        self.on_literal_number(value);
      }
      Token::Symbol => {
        let symbol = self.token_span();
        self.next();
        match self.token() {
          Token::Equal if is_stmt => {
            self.next();
            self.parse_expr();
            self.on_set(symbol);
            return true;
          }
          Token::Dec => {
            self.next();
            self.on_inc_dec_local(PrePost::Post, IncDec::Dec, symbol);
          }
          Token::Inc => {
            self.next();
            self.on_inc_dec_local(PrePost::Post, IncDec::Inc, symbol);
          }
          _ => {
            self.on_variable(symbol);
          }
        }
      }
      Token::Dec => {
        self.next();
        let s = self.expect_symbol();
        self.on_pre_op(s, Op1::Dec);
      }
      Token::Inc => {
        self.next();
        let s = self.expect_symbol();
        self.on_pre_op(s, Op1::Inc);
      }
      Token::Hyphen => {
        self.next();
        self.parse_expr_prec(P::Prefix);
        self.on_op1(Op1::Neg);
      }
      Token::Not => {
        self.next();
        self.parse_expr_prec(P::Prefix);
        self.on_op1(Op1::Not);
      }
      Token::If => {
        self.next();
        self.parse_expr();
        let n = self.parse_block();
        if self.token() == Token::Else {
          self.next();
          let m = self.parse_block();
          self.on_if_else(n, m);
        } else {
          self.on_if(n);
        }
      }
      Token::Loop => {
        self.next();
        let n = self.parse_block();
        self.on_loop(n);
      }
      _ => {
        self.on_error_missing_expr();
      }
    }

    loop {
      match self.token() {
        Token::Query if p <= P::Select => {
          self.next();
          self.parse_expr();
          self.expect(Token::Colon);
          self.parse_expr_prec(P::Select);
          self.on_select();
        }
        Token::Or if p < P::Or => {
          // TODO: right assoc
          self.next();
          self.parse_expr_prec(P::Or);
          self.on_or();
        }
        Token::And if p < P::And => {
          // TODO: right assoc
          self.next();
          self.parse_expr_prec(P::And);
          self.on_and();
        }
        Token::CmpEq if p < P::Cmp => {
          self.next();
          self.parse_expr_prec(P::Cmp);
          self.on_op2(Op2::CmpEq);
        }
        Token::CmpGe if p < P::Cmp => {
          self.next();
          self.parse_expr_prec(P::Cmp);
          self.on_op2(Op2::CmpGe);
        }
        Token::CmpGt if p < P::Cmp => {
          self.next();
          self.parse_expr_prec(P::Cmp);
          self.on_op2(Op2::CmpGt);
        }
        Token::CmpLe if p < P::Cmp => {
          self.next();
          self.parse_expr_prec(P::Cmp);
          self.on_op2(Op2::CmpLe);
        }
        Token::CmpLt if p < P::Cmp => {
          self.next();
          self.parse_expr_prec(P::Cmp);
          self.on_op2(Op2::CmpLt);
        }
        Token::CmpNe if p < P::Cmp => {
          self.next();
          self.parse_expr_prec(P::Cmp);
          self.on_op2(Op2::CmpNe);
        }
        Token::BitOr if p < P::BitOr => {
          self.next();
          self.parse_expr_prec(P::BitOr);
          self.on_op2(Op2::BitOr);
        }
        Token::BitXor if p < P::BitXor => {
          self.next();
          self.parse_expr_prec(P::BitXor);
          self.on_op2(Op2::BitXor);
        }
        Token::BitAnd if p < P::BitAnd => {
          self.next();
          self.parse_expr_prec(P::BitAnd);
          self.on_op2(Op2::BitAnd);
        }
        Token::Shl if p < P::Shift => {
          self.next();
          self.parse_expr_prec(P::Shift);
          self.on_op2(Op2::Shl);
        }
        Token::Shr if p < P::Shift => {
          self.next();
          self.parse_expr_prec(P::Shift);
          self.on_op2(Op2::Shr);
        }
        Token::Add if p < P::Add => {
          self.next();
          self.parse_expr_prec(P::Add);
          self.on_op2(Op2::Add);
        }
        Token::Hyphen if p < P::Add => {
          self.next();
          self.parse_expr_prec(P::Add);
          self.on_op2(Op2::Sub);
        }
        Token::Div if p < P::Mul => {
          self.next();
          self.parse_expr_prec(P::Mul);
          self.on_op2(Op2::Div);
        }
        Token::Mul if p < P::Mul => {
          self.next();
          self.parse_expr_prec(P::Mul);
          self.on_op2(Op2::Mul);
        }
        Token::Rem if p < P::Mul => {
          self.next();
          self.parse_expr_prec(P::Mul);
          self.on_op2(Op2::Rem);
        }
        Token::Field if self.token_is_attached() => {
          // TODO: not after block?
          // TODO: maybe define attachment to never include attaching to an
          // RBrace
          let symbol = &self.token_span()[1 ..];
          self.next();
          if is_stmt && self.token() == Token::Equal {
            self.next();
            self.parse_expr();
            self.on_set_field(symbol);
            return true;
          } else {
            self.on_get_field(symbol);
          }
        }
        Token::LBracket if self.token_is_attached() => {
          // TODO: not after block?
          self.next();
          self.parse_expr();
          self.expect(Token::RBracket);
          if is_stmt && self.token() == Token::Equal {
            self.next();
            self.parse_expr();
            self.on_set_index();
            return true;
          } else {
            self.on_get_index();
          }
        }
        Token::LParen if self.token_is_attached() => {
          // TODO: not after block?
          self.next();
          let n_args = self.parse_expr_list(Token::RParen);
          self.expect(Token::RParen);
          self.on_call(n_args);
        }
        _ => {
          return false;
        }
      }
    }
  }

  fn parse_block(&mut self) -> usize {
    self.expect(Token::LBrace);

    let mut n_stmts = 0;

    loop {
      match self.token() {
        Token::RBrace => {
          self.next();
          break;
        }
        Token::Break => {
          self.next();
          let n_args = self.parse_expr_list(Token::RBrace);
          self.on_break(n_args);
          n_stmts += 1;
          self.expect(Token::RBrace);
          break;
        }
        Token::Continue => {
          self.next();
          self.on_continue();
          n_stmts += 1;
          self.expect(Token::RBrace);
          break;
        }
        Token::Let => {
          self.next();
          // NB: we allow a list of zero bindings, like
          //
          //   let = f(x)
          //
          // which is a bit weird. but works semantically
          let n_bindings = self.parse_binding_list(Token::Equal);
          self.expect(Token::Equal);
          let mut n_exprs = 0;
          loop {
            self.parse_expr();
            n_exprs += 1;
            if self.token() != Token::Comma { break; }
            self.next();
          }
          self.on_let(n_bindings, n_exprs);
          n_stmts += 1;
        }
        Token::Return => {
          self.next();
          let n_args = self.parse_expr_list(Token::RBrace);
          self.on_return(n_args);
          n_stmts += 1;
          self.expect(Token::RBrace);
          break;
        }
        Token::Var => {
          self.next();
          let symbol = self.expect_symbol();
          self.expect(Token::Equal);
          self.parse_expr();
          self.on_variable(symbol);
          n_stmts += 1;
        }
        Token::While => {
          self.next();
          self.parse_expr();
          let n = self.parse_block();
          self.on_while(n);
          n_stmts += 1;
        }
        _ => {
          let pos = self.token_start();

          if ! self.parse_prec(P::Any, true) {
            let mut n_exprs = 1;
            while self.token() == Token::Comma {
              self.next();
              self.parse_expr();
              n_exprs += 1;
            }
            self.on_stmt_expr_list(n_exprs);
          }

          n_stmts += 1;

          if self.token_start() == pos {
            // NB: If we couldn't parse anything at all, then we immediately
            // close the block so that we don't get stuck in an infinite loop.
            //
            // Note that we already know that there ISN'T an RBrace here, so the
            // expect will fail.
            //
            // Also, note that in this case we still emit an `undefined`
            // expr/stmt.

            self.expect(Token::RBrace);
            break;
          }
        }
      }
    }

    return n_stmts;
  }

  */

  // ------- PARSER OUTPUT TO AST -------

  fn push_item(&mut self, x: Item<'a>) {
    self.items.push(x);
  }

  fn push_binding(&mut self, x: Binding<'a>) {
    self.bindings.push(x);
  }

  fn pop_binding_list(&mut self, n: usize) -> Box<[Binding<'a>]> {
    self.bindings.pop_list(n).collect()
  }

  fn push_expr(&mut self, x: Expr<'a>) {
    self.exprs.push(x);
  }

  fn pop_expr(&mut self) -> Expr<'a> {
    self.exprs.pop().unwrap()
  }

  fn pop_expr_list(&mut self, n: usize) -> Box<[Expr<'a>]> {
    self.exprs.pop_list(n).collect()
  }

  fn push_stmt(&mut self, x: Stmt<'a>) {
    self.stmts.push(x);
  }

  fn pop_stmt_list(&mut self, n: usize) -> Box<[Stmt<'a>]> {
    self.stmts.pop_list(n).collect()
  }

  fn on_fun(&mut self, name: &'a [u8], n_args: usize, n_stmts: usize) {
    let z = self.pop_stmt_list(n_stmts);
    let y = self.pop_binding_list(n_args);
    self.push_item(Item::Fun(Box::new(Fun { name, args: y, body: z })));
  }

  fn on_binding(&mut self, name: Option<&'a [u8]>) {
    let x = Binding { name };
    self.push_binding(x);
  }

  fn on_variable(&mut self, symbol: &'a [u8]) {
    self.push_expr(Expr::Variable(symbol));
  }

  fn on_literal_bool(&mut self, value: bool) {
    self.push_expr(Expr::ConstBool(value));
  }

  fn on_literal_int(&mut self, x: &'a [u8]) {
    let n =
      match i64::from_str_radix(str::from_utf8(x).unwrap(), 10) {
        Err(_) => {
          self.push_expr(Expr::Fail);
          return;
        }
        Ok(n) => n
      };
    self.push_expr(Expr::ConstI64(n));
  }

  fn on_select(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let p = self.pop_expr();
    self.push_expr(Expr::Select(Box::new((p, x, y))));
  }

  fn on_log_or(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    self.push_expr(Expr::LogOr(Box::new((x, y))));
  }

  fn on_log_and(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    self.push_expr(Expr::LogAnd(Box::new((x, y))));
  }

  fn on_op1(&mut self, op: Op1) {
    let x = self.pop_expr();
    self.push_expr(Expr::Op1(Box::new((op, x))));
  }

  fn on_op2(&mut self, op: Op2) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    self.push_expr(Expr::Op2(Box::new((op, x, y))));
  }

  fn on_inc_dec_local(&mut self, p: PrePost, op: IncDec, symbol: &'a [u8]) {
    self.push_expr(Expr::IncDecLocal(Box::new((p, op, symbol))));
  }

  fn on_get_field(&mut self, symbol: &'a [u8]) {
    let x = self.pop_expr();
    self.push_expr(Expr::GetField(Box::new((x, symbol))));
  }

  fn on_get_index(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    self.push_expr(Expr::GetIndex(Box::new((x, y))));
  }

  fn on_if(&mut self, n_stmts: usize) {
    let y = self.pop_stmt_list(n_stmts);
    let x = self.pop_expr();
    self.push_expr(Expr::If(Box::new((x, y))));
  }

  fn on_if_else(&mut self, n_stmts_then: usize, n_stmts_else: usize) {
    let z = self.pop_stmt_list(n_stmts_else);
    let y = self.pop_stmt_list(n_stmts_then);
    let x = self.pop_expr();
    self.push_expr(Expr::IfElse(Box::new((x, y, z))));
  }

  fn on_call(&mut self, n_args: usize) {
    let x = self.pop_expr_list(n_args);
    let f = self.pop_expr();
    self.push_expr(Expr::Call(Box::new((f, x))));
  }

  fn on_loop(&mut self, n_stmts: usize) {
    let x = self.pop_stmt_list(n_stmts);
    self.push_expr(Expr::Loop(x));
  }

  fn on_stmt_expr_list(&mut self, n_exprs: usize) {
    let x = self.pop_expr_list(n_exprs);
    self.push_stmt(Stmt::ExprList(x));
  }

  fn on_break(&mut self, n_args: usize) {
    let x = self.pop_expr_list(n_args);
    self.push_stmt(Stmt::Break(x));
  }

  fn on_continue(&mut self) {
    self.push_stmt(Stmt::Continue);
  }

  fn on_let(&mut self, n_bindings: usize, n_exprs: usize) {
    let y = self.pop_expr_list(n_exprs);
    let x = self.pop_binding_list(n_bindings);
    self.push_stmt(Stmt::Let(x, y));
  }

  fn on_return(&mut self, n_args: usize) {
    let x = self.pop_expr_list(n_args);
    self.push_stmt(Stmt::Ret(x));
  }

  fn on_set_local(&mut self, symbol: &'a [u8]) {
    let x = self.pop_expr();
    self.push_stmt(Stmt::SetLocal(symbol, x));
  }

  fn on_set_field(&mut self, symbol: &'a [u8]) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    self.push_stmt(Stmt::SetField(x, symbol, y));
  }

  fn on_set_index(&mut self) {
    let z = self.pop_expr();
    let y = self.pop_expr();
    let x = self.pop_expr();
    self.push_stmt(Stmt::SetIndex(x, y, z));
  }

  fn on_let_mutable(&mut self, symbol: &'a [u8]) {
    let x = self.pop_expr();
    self.push_stmt(Stmt::LetMutable(symbol, x));
  }

  fn on_while(&mut self, n_stmts: usize) {
    let y = self.pop_stmt_list(n_stmts);
    let x = self.pop_expr();
    self.push_stmt(Stmt::While(x, y));
  }

  fn on_error_missing_expected_token(&mut self, token: Token) {
    let _ = token;
    // TODO: report error on missing expected token
  }

  fn on_error_missing_expr(&mut self) {
    // TODO: report error on missing expected expression
    self.push_expr(Expr::Fail);
  }
}

use crate::ast::Bind;
use crate::ast::Expr;
use crate::ast::Fun;
use crate::ast::Item;
use crate::ast::Op1;
use crate::ast::Op2;
use crate::ast::Stmt;
use crate::lexer::scan;
use crate::prelude::*;
use crate::symbol::Symbol;
use crate::symbol::SymbolTable;
use crate::token::Token;

// TODO: consider defining a token immediately following a "}" as never attached

pub fn parse(source: &[u8]) -> Box<[Item]> {
  let source = scan(source);
  let mut t = Parser::new(&source);
  t.parse_item_list();
  t.items.into_boxed_slice()
}

struct Parser<'a, 'b> {
  source: &'b [(Token, bool, &'a [u8])],
  token_index: usize,
  token: Token,
  token_is_attached: bool,
  token_span: &'a [u8],
  items: Vec<Item>,
  binds: Vec<Bind>,
  exprs: Vec<Expr>,
  stmts: Vec<Stmt>,
  symbol_table: SymbolTable,
  error: Option<ParseError>,
}

#[derive(Clone, Copy, Debug)]
enum ParseError {
  ExpectedAttachedSymbol,
  ExpectedEqual,
  ExpectedExpression,
  ExpectedLBrace,
  ExpectedLParen,
  ExpectedName,
  ExpectedRBrace,
  ExpectedRBracket,
  ExpectedRParen,
  MalformedInt,
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum Priority {
  Any,
  ShortCircuitOr,
  ShortCircuitAnd,
  Cmp,
  Add,
  Mul,
  Unary,
}

fn next<'a, 'b>(source: &'b [(Token, bool, &'a [u8])])
  -> ((Token, bool, &'a [u8]), &'b [(Token, bool, &'a [u8])])
{
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
      binds: Vec::new(),
      exprs: Vec::new(),
      stmts: Vec::new(),
      symbol_table: SymbolTable::new(),
      error: None,
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

  fn token_str(&self) -> &'a str {
    str::from_utf8(self.token_span).unwrap()
  }

  fn token_symbol(&mut self) -> Symbol {
    self.symbol_table.intern(self.token_str())
  }

  fn on_error(&mut self, e: ParseError) {
    // TODO: mark error location
    if let None = self.error { self.error = Some(e); }
  }

  fn parse_token(&mut self, token: Token, error: ParseError) {
    if self.token() != token {
      self.on_error(error);
    } else {
      self.next();
    }
  }

  fn parse_item_list(&mut self) {
    loop {
      match self.token() {
        Token::Eof => break,
        Token::Fun => {
          if let Err(_) = self.parse_fun() { break; }
        }
        _ => {
          // TODO: error?
          break;
        }
      }
    }
  }

  fn parse_equal(&mut self) {
    self.parse_token(Token::Equal, ParseError::ExpectedEqual)
  }

  fn parse_lparen(&mut self) {
    self.parse_token(Token::LParen, ParseError::ExpectedLParen)
  }

  fn parse_rparen(&mut self) {
    self.parse_token(Token::RParen, ParseError::ExpectedRParen)
  }

  fn parse_rbracket(&mut self) {
    self.parse_token(Token::RBracket, ParseError::ExpectedRBracket)
  }

  fn parse_lbrace(&mut self) {
    self.parse_token(Token::LBrace, ParseError::ExpectedLBrace)
  }

  fn parse_rbrace(&mut self) {
    self.parse_token(Token::RBrace, ParseError::ExpectedRBrace)
  }

  fn parse_path_segment(&mut self) -> Symbol {
    if self.token() == Token::Symbol && self.token_is_attached() {
      let s = self.token_symbol();
      self.next();
      s
    } else {
      self.on_error(ParseError::ExpectedAttachedSymbol);
      self.symbol_table.intern("_")
    }
  }

  fn parse_name(&mut self) -> Option<Symbol> {
    match self.token() {
      Token::Symbol => {
        let s = self.token_symbol();
        self.next();
        Some(s)
      }
      Token::Underscore => {
        self.next();
        None
      }
      _ => {
        self.on_error(ParseError::ExpectedName);
        None
      }
    }
  }

  fn parse_fun(&mut self) -> Result<(), ParseError> {
    debug_assert!(self.token() == Token::Fun);
    self.next();
    let name = self.parse_name();
    self.parse_lparen();
    let m = self.parse_bind_list(Token::RParen);
    self.parse_rparen();
    let n = self.parse_block();
    self.on_fun(name, m, n);
    let e = replace(&mut self.error, None);
    match e { None => Ok(()), Some(e) => Err(e), }
  }

  fn parse_bind_list(&mut self, stop: Token) -> usize {
    let mut n_binds = 0;
    if self.token() != stop {
      loop {
        let name = self.parse_name();
        self.on_bind(name);
        n_binds += 1;
        if self.token() != Token::Comma { break; }
        self.next();
      }
    }
    n_binds
  }

  fn parse_expr(&mut self) {
    self.parse_expr_prec(Priority::Any);
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
    n_exprs
  }

  fn parse_expr_prec(&mut self, p: Priority) {
    let is_stmt: bool = self.parse_prec(p, false);
    assert!(! is_stmt);
  }

  // returns `true` if we parsed a statement

  fn parse_prec(&mut self, p: Priority, is_stmt: bool) -> bool {
    match self.token() {
      Token::LParen => {
        self.next();
        self.parse_expr();
        self.parse_rparen();
      }
      Token::True => {
        self.next();
        self.on_literal_bool(true);
      }
      Token::False => {
        self.next();
        self.on_literal_bool(false);
      }
      Token::Int => {
        let value = self.token_str();
        self.next();
        self.on_literal_int(value);
      }
      Token::Symbol => {
        let symbol = self.token_symbol();
        self.next();
        match self.token() {
          Token::Equal if is_stmt => {
            self.next();
            self.parse_expr();
            self.on_set_local(symbol);
            return true;
          }
          Token::AddEqual if is_stmt => {
            self.next();
            self.parse_expr();
            self.on_set_local_compound(symbol, Op2::Add);
            return true;
          }
          Token::SubEqual if is_stmt => {
            self.next();
            self.parse_expr();
            self.on_set_local_compound(symbol, Op2::Add);
            return true;
          }
          _ => {
            self.on_variable(symbol);
          }
        }
      }
      Token::BitXor => {
        self.next();
        self.parse_expr_prec(Priority::Unary);
        self.on_op1(Op1::BitComplement);
      }
      Token::Sub => {
        self.next();
        self.parse_expr_prec(Priority::Unary);
        self.on_op1(Op1::Neg);
      }
      Token::Not => {
        self.next();
        self.parse_expr_prec(Priority::Unary);
        self.on_op1(Op1::Not);
      }
      Token::If => {
        self.next();
        self.parse_lparen();
        self.parse_expr();
        self.parse_rparen();
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
        self.on_fail_expr(ParseError::ExpectedExpression);
        return false;
      }
    }

    loop {
      match self.token() {
        Token::ShortCircuitOr if p <= Priority::ShortCircuitOr => {
          self.next();
          self.parse_expr_prec(Priority::ShortCircuitOr);
          self.on_short_circuit_or();
        }
        Token::ShortCircuitAnd if p <= Priority::ShortCircuitAnd => {
          self.next();
          self.parse_expr_prec(Priority::ShortCircuitAnd);
          self.on_short_circuit_and();
        }
        Token::CmpEq if p < Priority::Cmp => {
          self.next();
          self.parse_expr_prec(Priority::Cmp);
          self.on_op2(Op2::CmpEq);
        }
        Token::CmpGe if p < Priority::Cmp => {
          self.next();
          self.parse_expr_prec(Priority::Cmp);
          self.on_op2(Op2::CmpGe);
        }
        Token::CmpGt if p < Priority::Cmp => {
          self.next();
          self.parse_expr_prec(Priority::Cmp);
          self.on_op2(Op2::CmpGt);
        }
        Token::CmpLe if p < Priority::Cmp => {
          self.next();
          self.parse_expr_prec(Priority::Cmp);
          self.on_op2(Op2::CmpLe);
        }
        Token::CmpLt if p < Priority::Cmp => {
          self.next();
          self.parse_expr_prec(Priority::Cmp);
          self.on_op2(Op2::CmpLt);
        }
        Token::CmpNe if p < Priority::Cmp => {
          self.next();
          self.parse_expr_prec(Priority::Cmp);
          self.on_op2(Op2::CmpNe);
        }
        Token::BitOr if p < Priority::Add => {
          self.next();
          self.parse_expr_prec(Priority::Add);
          self.on_op2(Op2::BitOr);
        }
        Token::BitXor if p < Priority::Add => {
          self.next();
          self.parse_expr_prec(Priority::Add);
          self.on_op2(Op2::BitXor);
        }
        Token::BitAnd if p < Priority::Mul => {
          self.next();
          self.parse_expr_prec(Priority::Mul);
          self.on_op2(Op2::BitAnd);
        }
        Token::Shl if p < Priority::Mul => {
          self.next();
          self.parse_expr_prec(Priority::Mul);
          self.on_op2(Op2::Shl);
        }
        Token::Shr if p < Priority::Mul => {
          self.next();
          self.parse_expr_prec(Priority::Mul);
          self.on_op2(Op2::Shr);
        }
        Token::Add if p < Priority::Add => {
          self.next();
          self.parse_expr_prec(Priority::Add);
          self.on_op2(Op2::Add);
        }
        Token::Sub if p < Priority::Add => {
          self.next();
          self.parse_expr_prec(Priority::Add);
          self.on_op2(Op2::Sub);
        }
        Token::Div if p < Priority::Mul => {
          self.next();
          self.parse_expr_prec(Priority::Mul);
          self.on_op2(Op2::Div);
        }
        Token::Mul if p < Priority::Mul => {
          self.next();
          self.parse_expr_prec(Priority::Mul);
          self.on_op2(Op2::Mul);
        }
        Token::Rem if p < Priority::Mul => {
          self.next();
          self.parse_expr_prec(Priority::Mul);
          self.on_op2(Op2::Rem);
        }
        Token::Dot if self.token_is_attached() => {
          self.next();
          let field = self.parse_path_segment();
          if is_stmt && self.token() == Token::Equal {
            self.next();
            self.parse_expr();
            self.on_set_field(field);
            return true;
          } else {
            self.on_get_field(field);
          }
        }
        Token::LBracket if self.token_is_attached() => {
          self.next();
          self.parse_expr();
          self.parse_rbracket();
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
          self.next();
          let n_args = self.parse_expr_list(Token::RParen);
          self.parse_rparen();
          self.on_call(n_args);
        }
        _ => {
          return false;
        }
      }
    }
  }

  fn parse_block(&mut self) -> usize {
    self.parse_lbrace();

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
          self.parse_rbrace();
          break;
        }
        Token::Continue => {
          self.next();
          self.on_continue();
          n_stmts += 1;
          self.parse_rbrace();
          break;
        }
        Token::Let => {
          self.next();
          if self.token() == Token::Mut {
            self.next();
            let name = self.parse_name();
            self.parse_equal();
            self.parse_expr();
            self.on_let_mut(name);
            n_stmts += 1;
          } else {
            // NB: we allow a list of zero binds, like
            //
            //   let = f(x)
            //
            // which is a bit weird. but works semantically
            let n_binds = self.parse_bind_list(Token::Equal);
            self.parse_equal();
            let mut n_exprs = 0;
            loop {
              self.parse_expr();
              n_exprs += 1;
              if self.token() != Token::Comma { break; }
              self.next();
            }
            self.on_let(n_binds, n_exprs);
            n_stmts += 1;
          }
        }
        Token::Return => {
          self.next();
          let n_args = self.parse_expr_list(Token::RBrace);
          self.on_return(n_args);
          n_stmts += 1;
          self.parse_rbrace();
          break;
        }
        Token::While => {
          self.next();
          self.parse_lparen();
          self.parse_expr();
          self.parse_rparen();
          let n = self.parse_block();
          self.on_while(n);
          n_stmts += 1;
        }
        _ => {
          let stmt_start = self.token_index();

          if ! self.parse_prec(Priority::Any, true) {
            let mut n_exprs = 1;
            while self.token() == Token::Comma {
              self.next();
              self.parse_expr();
              n_exprs += 1;
            }
            self.on_stmt_expr_list(n_exprs);
          }

          n_stmts += 1;

          if self.token_index() == stmt_start {
            // NB: If we couldn't parse anything at all, then we immediately
            // close the block so that we don't get stuck in an infinite loop.
            //
            // Note that we already know that there ISN'T an RBrace here, so the
            // expect will fail.
            //
            // Also, note that in this case we still emit a `fail` expr/stmt.

            self.parse_rbrace();
            break;
          }
        }
      }
    }

    n_stmts
  }

  // ------- PARSER OUTPUT TO AST -------

  fn push_item(&mut self, x: Item) {
    self.items.push(x);
  }

  fn push_bind(&mut self, x: Bind) {
    self.binds.push(x);
  }

  fn pop_bind_list(&mut self, n: usize) -> Box<[Bind]> {
    self.binds.pop_list(n).collect()
  }

  fn push_expr(&mut self, x: Expr) {
    self.exprs.push(x);
  }

  fn pop_expr(&mut self) -> Expr {
    self.exprs.pop().unwrap()
  }

  fn pop_expr_list(&mut self, n: usize) -> Box<[Expr]> {
    self.exprs.pop_list(n).collect()
  }

  fn push_stmt(&mut self, x: Stmt) {
    self.stmts.push(x);
  }

  fn pop_stmt_list(&mut self, n: usize) -> Box<[Stmt]> {
    self.stmts.pop_list(n).collect()
  }

  fn on_fun(&mut self, name: Option<Symbol>, n_args: usize, n_stmts: usize) {
    let z = self.pop_stmt_list(n_stmts);
    let y = self.pop_bind_list(n_args);
    self.push_item(Item::Fun(Box::new(Fun { name, args: y, body: z })));
  }

  fn on_bind(&mut self, name: Option<Symbol>) {
    self.push_bind(Bind { name });
  }

  fn on_variable(&mut self, symbol: Symbol) {
    self.push_expr(Expr::Variable(symbol));
  }

  fn on_literal_bool(&mut self, value: bool) {
    self.push_expr(Expr::LiteralBool(value));
  }

  fn on_literal_int(&mut self, x: &str) {
    match i64::from_str_radix(x, 10) {
      Err(_) =>
        self.on_fail_expr(ParseError::MalformedInt),
      Ok(n) =>
        self.push_expr(Expr::LiteralI64(n)),
    }
  }

  fn on_short_circuit_or(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    self.push_expr(Expr::ShortCircuitOr(Box::new((x, y))));
  }

  fn on_short_circuit_and(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    self.push_expr(Expr::ShortCircuitAnd(Box::new((x, y))));
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

  fn on_get_field(&mut self, symbol: Symbol) {
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

  fn on_let(&mut self, n_binds: usize, n_exprs: usize) {
    let y = self.pop_expr_list(n_exprs);
    let x = self.pop_bind_list(n_binds);
    self.push_stmt(Stmt::Let(x, y));
  }

  fn on_return(&mut self, n_args: usize) {
    let x = self.pop_expr_list(n_args);
    self.push_stmt(Stmt::Ret(x));
  }

  fn on_set_local(&mut self, symbol: Symbol) {
    let x = self.pop_expr();
    self.push_stmt(Stmt::SetLocal(symbol, x));
  }

  fn on_set_local_compound(&mut self, symbol: Symbol, op: Op2) {
    let x = self.pop_expr();
    self.push_stmt(Stmt::SetLocalCompound(symbol, op, x));
  }

  fn on_set_field(&mut self, symbol: Symbol) {
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

  fn on_let_mut(&mut self, name: Option<Symbol>) {
    let x = self.pop_expr();
    self.push_stmt(Stmt::DefLocal(name, x));
  }

  fn on_while(&mut self, n_stmts: usize) {
    let y = self.pop_stmt_list(n_stmts);
    let x = self.pop_expr();
    self.push_stmt(Stmt::While(x, y));
  }

  fn on_fail_expr(&mut self, error: ParseError) {
    self.on_error(error);
    self.push_expr(Expr::Fail);
  }
}

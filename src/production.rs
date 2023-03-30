use std::fmt::Write;

use super::symbol::{Symbol,SymbolDb};

#[derive(Clone,Debug,Eq,Hash,PartialEq,PartialOrd,Ord)]
pub struct Production {
    lhs: Symbol,
    rhs: Vec<Symbol>,
}

impl Production {
  pub fn new(lhs: Symbol, rhs: Vec<Symbol>) -> Production {
      Production { lhs, rhs }
  }

  pub fn lhs(&self) -> &Symbol { &self.lhs }
  pub fn rhs(&self) -> &Vec<Symbol> { &self.rhs }

  pub fn to_string(&self, symbol_db: &SymbolDb) -> String {
      let mut result = String::new();
      write!(&mut result, "{} -> ", symbol_db.label(&self.lhs).unwrap()).unwrap();
      let mut iter = self.rhs.iter().peekable();
      while let Some(s) = iter.next() {
          write!(&mut result, "{}", symbol_db.label(s).unwrap()).unwrap();
          if let Some(_) = iter.peek() {
              write!(&mut result, "  ").unwrap();
          }
      }
      result
  }
}


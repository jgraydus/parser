use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Error;

use super::production::Production;
use super::symbol::{Symbol,SymbolDb};

#[derive(Debug)]
pub struct Grammar {
    symbol_db: SymbolDb,
    start_symbol: Symbol,
    productions: HashMap<Symbol, Vec<Production>>,
}

impl Grammar {
  pub fn new(symbol_db: SymbolDb, start_symbol: Symbol, productions: Vec<Production>) -> Grammar {
      let mut productions = productions;

      // add the rule "goal -> start_symbol $"
      let p = Production::new(symbol_db.goal(), vec![start_symbol, symbol_db.eoi()]);
      productions.push(p);

      fn group_by_lhs(ps: &Vec<Production>) -> HashMap<Symbol,Vec<Production>> {
          let mut result: HashMap<Symbol,Vec<Production>> = HashMap::new();
          ps.iter().for_each(|p| {
              let lhs = p.lhs();
              if !result.contains_key(lhs) {
                  result.insert(*lhs, Vec::new());
              }
              result.get_mut(lhs).unwrap().push(p.clone());
          });
          result
      }

      Grammar {
          symbol_db,
          start_symbol,
          productions: group_by_lhs(&productions),
      }
  }

  pub fn start_symbol(&self) -> &Symbol { &self.start_symbol }
  pub fn productions(&self, lhs: &Symbol) -> Option<&Vec<Production>> { self.productions.get(lhs) }
  pub fn terminals(&self) -> &HashSet<Symbol> { &self.symbol_db.terminals() }
  pub fn nonterminals(&self) -> &HashSet<Symbol> { &self.symbol_db.non_terminals() }
  pub fn symbol_db(&self) -> &SymbolDb { &self.symbol_db }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let symbol_db = &self.symbol_db;
        write!(f, "grammar:\n")?;
        let s = symbol_db.label(&self.start_symbol()).ok_or(Error)?;
        write!(f, "  start symbol = {}\n", s)?;
        write!(f, "  productions =\n")?;
        for (_,v) in &self.productions {
            for p in v {
                write!(f, "    {}\n", p.to_string(symbol_db))?;
            }
        }
        Ok(())
    }
}


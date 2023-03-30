use std::fmt::Write;
use super::production::Production;
use super::symbol::{Symbol,SymbolDb};

#[derive(Clone,Debug,Eq,Hash,PartialEq,PartialOrd,Ord)]
pub struct LR1Item {
    production: Production,
    dot_position: usize,
    lookahead: Symbol,
}

impl LR1Item {
    pub fn new(production: Production, dot_position: usize, lookahead: Symbol) -> LR1Item {
        LR1Item { production, dot_position, lookahead }
    }

    pub fn production(&self) -> &Production {
        &self.production
    }

    pub fn dot_position(&self) -> usize {
        self.dot_position
    }

    pub fn symbols_after_dot(&self) -> Vec<Symbol> {
        let pos = self.dot_position;
        let s = &self.production.rhs()[pos..];
        let mut result = Vec::new();
        result.extend_from_slice(s);
        result
    }

    pub fn lookahead(&self) -> &Symbol {
        &self.lookahead
    }

    pub fn is_target(&self, symbol_db: &SymbolDb) -> bool {
        self.production.lhs() == &symbol_db.goal() && self.lookahead() == &symbol_db.eoi()
    }

    #[allow(dead_code)]
    pub fn to_string(&self, symbol_db: &SymbolDb) -> String {
        let mut result = String::new();
        let p = self.production.to_string(symbol_db);
        let d = self.dot_position;
        let l = symbol_db.label(&self.lookahead).unwrap();
        write!(&mut result, "[LR1Item {}, {}, {}]", p, d, l).unwrap();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_target_01() {
        let mut symbol_db = SymbolDb::new();
        let a = symbol_db.new_terminal("a");
        let p = Production::new(symbol_db.goal(), vec![a]);
        let item = LR1Item::new(p, 0, symbol_db.eoi());
        assert!(item.is_target(&symbol_db));
    }

    #[test]
    fn is_target_02() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("s");
        let a = symbol_db.new_terminal("a");
        let p = Production::new(s, vec![a]);
        let item = LR1Item::new(p, 0, symbol_db.eoi());
        assert!(!item.is_target(&symbol_db));
    }

    #[test]
    fn is_target_03() {
        let mut symbol_db = SymbolDb::new();
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let p = Production::new(symbol_db.goal(), vec![a]);
        let item = LR1Item::new(p, 0, b);
        assert!(!item.is_target(&symbol_db));
    }

    #[test]
    fn symbols_after_dot_01() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("s");
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let c = symbol_db.new_terminal("c");
        let d = symbol_db.new_terminal("d");
        let e = symbol_db.new_terminal("e");
        let p = Production::new(s, vec![a, b, c, d, e]);
        let item = LR1Item::new(p, 0, e);
        let result = item.symbols_after_dot();
        assert_eq!(result, vec![a, b, c, d, e]);
    }

    #[test]
    fn symbols_after_dot_02() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("s");
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let c = symbol_db.new_terminal("c");
        let d = symbol_db.new_terminal("d");
        let e = symbol_db.new_terminal("e");
        let p = Production::new(s, vec![a, b, c, d, e]);
        let item = LR1Item::new(p, 1, e);
        let result = item.symbols_after_dot();
        assert_eq!(result, vec![b, c, d, e]);
    }

    #[test]
    fn symbols_after_dot_03() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("s");
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let c = symbol_db.new_terminal("c");
        let d = symbol_db.new_terminal("d");
        let e = symbol_db.new_terminal("e");
        let p = Production::new(s, vec![a, b, c, d, e]);
        let item = LR1Item::new(p, 2, e);
        let result = item.symbols_after_dot();
        assert_eq!(result, vec![c, d, e]);
    }

    #[test]
    fn symbols_after_dot_04() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("s");
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let c = symbol_db.new_terminal("c");
        let d = symbol_db.new_terminal("d");
        let e = symbol_db.new_terminal("e");
        let p = Production::new(s, vec![a, b, c, d, e]);
        let item = LR1Item::new(p, 3, e);
        let result = item.symbols_after_dot();
        assert_eq!(result, vec![d, e]);
    }

    #[test]
    fn symbols_after_dot_05() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("s");
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let c = symbol_db.new_terminal("c");
        let d = symbol_db.new_terminal("d");
        let e = symbol_db.new_terminal("e");
        let p = Production::new(s, vec![a, b, c, d, e]);
        let item = LR1Item::new(p, 4, e);
        let result = item.symbols_after_dot();
        assert_eq!(result, vec![e]);
    }

    #[test]
    fn symbols_after_dot_06() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("s");
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let c = symbol_db.new_terminal("c");
        let d = symbol_db.new_terminal("d");
        let e = symbol_db.new_terminal("e");
        let p = Production::new(s, vec![a, b, c, d, e]);
        let item = LR1Item::new(p, 5, e);
        let result = item.symbols_after_dot();
        assert_eq!(result, vec![]);
    }
}


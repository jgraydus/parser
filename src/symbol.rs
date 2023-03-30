use std::collections::{HashMap,HashSet};

#[derive(Clone,Copy,Debug,Eq,Hash,Ord,PartialOrd,PartialEq)]
pub struct Symbol(usize);

#[derive(Debug)]
pub struct SymbolDb {
    next: usize,
    from_label: HashMap<String,Symbol>,
    to_label: HashMap<Symbol,String>,
    terminals: HashSet<Symbol>,
    non_terminals: HashSet<Symbol>,
}

impl SymbolDb {
    pub fn new() -> SymbolDb {
        let mut s = SymbolDb {
            next: 0,
            from_label: HashMap::new(),
            to_label: HashMap::new(),
            terminals: HashSet::new(),
            non_terminals: HashSet::new(),
        };
        s.new_nonterminal("GOAL");
        s.new_terminal("$");
        s.new_terminal("ε");
        s
    }

    fn new_symbol(&mut self, label: &str) -> Symbol {
        if self.from_label.contains_key(label) {
            panic!("the symbol [{}] is already defined", label);
        }
        let s = Symbol(self.next);
        self.next = self.next + 1;
        self.from_label.insert(label.to_string(), s);
        self.to_label.insert(s, label.to_string());
        s
    }

    pub fn new_nonterminal(&mut self, label: &str) -> Symbol {
        let s = self.new_symbol(label);
        self.non_terminals.insert(s);
        s
    }

    pub fn new_terminal(&mut self, label: &str) -> Symbol {
        let s = self.new_symbol(label);
        self.terminals.insert(s);
        s
    }

    pub fn is_terminal(&self, s: &Symbol) -> bool {
        self.terminals.contains(s)
    }

    pub fn epsilon(&self) -> Symbol {
        self.from_label.get("ε").expect("missing epsilon symbol").clone()
    }

    pub fn goal(&self) -> Symbol {
        self.from_label.get("GOAL").expect("missing goal symbol").clone()
    }

    pub fn eoi(&self) -> Symbol {
        self.from_label.get("$").expect("missing end of input symbol").clone()
    }

    pub fn terminals(&self) -> &HashSet<Symbol> {
        &self.terminals
    }

    pub fn non_terminals(&self) -> &HashSet<Symbol> {
        &self.non_terminals
    }

    pub fn label(&self, s: &Symbol) -> Option<&String> {
        self.to_label.get(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_db_01() {
        let db = SymbolDb::new();
        db.epsilon();
        db.goal();
        db.eoi();
    }

    #[test]
    fn symbol_db_02() {
        let mut db = SymbolDb::new();
        let s = db.new_nonterminal("foo");
        assert!(!db.is_terminal(&s));
    }

    #[test]
    fn symbol_db_03() {
        let mut db = SymbolDb::new();
        let s = db.new_terminal("foo");
        assert!(db.is_terminal(&s));
    }
}


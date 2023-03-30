use std::collections::HashMap;
use std::fmt::Write;

use super::action::Action;
use super::canonical_collection::CanonicalCollection;
use super::grammar::Grammar;
use super::symbol::{Symbol,SymbolDb};

#[derive(Debug)]
pub struct ParseTables {
    action_table: HashMap<(u32,Symbol),Action>,
    goto_table: HashMap<(u32,Symbol),u32>
}

impl ParseTables {
    pub fn new(grammar: &Grammar) -> ParseTables {
        build(grammar)
    }

    pub fn action(&self, state: u32, symbol: Symbol) -> Option<&Action> {
        let key = (state, symbol);
        self.action_table.get(&key)
    }

    pub fn transition(&self, state: u32, symbol: Symbol) -> Option<&u32> {
        let key = (state, symbol);
        self.goto_table.get(&key)
    }

    fn add_action(&mut self, state: u32, symbol: Symbol, action: Action) {
        let key = (state, symbol);
        if let Some(other) = self.action_table.get(&key) {
            if other == &action {
                return;
            }
            match (&action, &other) {
                (Action::Shift(_), Action::Reduce(_)) => {
                    println!("shift/reduce conflict");
                    self.action_table.insert(key, action);
                },
                (Action::Reduce(_), Action::Shift(_)) => {
                    println!("shift/reduce conflict");
                },
                (Action::Reduce(_), Action::Reduce(_)) => {
                    panic!("reduce/reduce conflict");
                },
                (x,y) => panic!("unknown conflict -- {:?} {:?} {:?} {:?}", x, y, state, symbol)
            }
        } else {
            self.action_table.insert(key, action);
        }
    }

    fn add_transition(&mut self, from: u32, on: Symbol, to: u32) {
        let key = (from, on);
        if let Some(_) = self.goto_table.get(&key) {
            panic!("attempt to replace an existing entry in goto table");
        } else {
            self.goto_table.insert(key, to);
        }
    }

    #[allow(dead_code)]
    pub fn to_string(&self, symbol_db: &SymbolDb) -> String {
        let mut result = String::new();
        writeln!(&mut result, "actions").unwrap();
        for ((i,s), a) in &self.action_table {
            let s = symbol_db.label(&s).unwrap();
            let a = a.to_string(symbol_db);
            writeln!(&mut result, "    ({}, {}) -> {}", i, s, a).unwrap();
        }
        writeln!(&mut result, "goto").unwrap();
        for ((i,s), n) in &self.goto_table {
            let s = symbol_db.label(&s).unwrap();
            writeln!(&mut result, "    ({}, {}) -> {}", i, s, n).unwrap();
        }
        result
    }
}

fn build(grammar: &Grammar) -> ParseTables {
    let symbol_db = grammar.symbol_db();
    let cc = CanonicalCollection::new(grammar);

    let mut parse_tables = ParseTables {
        action_table: HashMap::new(),
        goto_table: HashMap::new()
    };

    for (&i, cc_i) in cc.sets() {
        for item in cc_i {
            let unseen = item.symbols_after_dot();
            // if the dot isn't at the end of the production (i.e. unseen isn't empty), and
            // this isn't an epsilon production, and there is a transition from the current state
            // on the next symbol of the production
            if !unseen.is_empty() &&
               unseen[0] != symbol_db.epsilon() &&
               cc.transitions().contains_key(&(i,unseen[0])) {
                let c = unseen[0];
                // if the next symbol is a terminal, then add a shift action
                if symbol_db.is_terminal(&c) {
                    let j = cc.transitions().get(&(i,c)).unwrap();
                    parse_tables.add_action(i, c, Action::shift(*j));
                }
            }
            // if there are no unseen symbols and the production represents the target, then add an
            // accept action
            else if unseen.is_empty() && item.is_target(grammar.symbol_db()) {
                parse_tables.add_action(i, symbol_db.eoi(), Action::accept());
            }
            // if at the end of a production rule or it's an epsilon production, then add a reduce action 
            else if unseen.is_empty() || unseen[0] == symbol_db.epsilon() {
                let action = Action::reduce(item.production().clone());
                //println!("**** {} {}     {}", i, symbol_db.label(item.lookahead()).unwrap(), item.to_string(symbol_db));
                parse_tables.add_action(i, item.lookahead().clone(), action);
            }
            else {
                panic!("something went terribly wrong while building parse tables");
            }
        }
        // add transitions for the non-terminals
        for nt in grammar.nonterminals() {
            if let Some(&j) = cc.transitions().get(&(i, nt.clone())) {
                parse_tables.add_transition(i, nt.clone(), j);
            } else {
                //println!("there is no transition from {} on a reduction to {}", i, nt);
            }
        }
    }

    parse_tables
}


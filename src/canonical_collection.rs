use std::collections::{BTreeMap,BTreeSet,HashMap,HashSet};

use super::grammar::Grammar;
use super::lr1_item::LR1Item;
use super::first_and_follow::FirstAndFollow;
use super::production::Production;
use super::symbol::Symbol;

#[derive(Debug)]
pub struct CanonicalCollection {
    next_number: u32,
    int_to_set: BTreeMap<u32,BTreeSet<LR1Item>>,
    set_to_int: BTreeMap<BTreeSet<LR1Item>,u32>,
    transitions: HashMap<(u32,Symbol),u32>,
    unprocessed: Vec<BTreeSet<LR1Item>>
}

impl CanonicalCollection {
    pub fn new(grammar: &Grammar) -> CanonicalCollection {
        build(grammar)
    }

    pub fn contains(&self, set: &BTreeSet<LR1Item>) -> bool {
        self.set_to_int.contains_key(set)
    }

    pub fn sets(&self) -> &BTreeMap<u32,BTreeSet<LR1Item>> {
        &self.int_to_set
    }

    pub fn transitions(&self) -> &HashMap<(u32,Symbol),u32> {
        &self.transitions
    }

    pub fn take_unprocessed(&mut self) -> Vec<BTreeSet<LR1Item>> {
        std::mem::replace(&mut self.unprocessed, Vec::new())
    }

    fn add(&mut self, set: BTreeSet<LR1Item>) {
        if self.set_to_int.contains_key(&set) {
            panic!("set is already in CC")
        }
        let n = self.next_number;
        self.set_to_int.insert(set.clone(), n);
        self.int_to_set.insert(n, set.clone());
        self.unprocessed.push(set);
        self.next_number = n + 1;
    }

    fn add_transition(&mut self, from: BTreeSet<LR1Item>, on: Symbol, to: BTreeSet<LR1Item>) {
        if !self.set_to_int.contains_key(&from) {
            panic!("[from] not in CC: {:?}", from);
        }
        if !self.set_to_int.contains_key(&to) {
            panic!("[to] not in CC: {:?}", to);
        }
        let from_n = *self.set_to_int.get(&from).unwrap();
        let to_n = *self.set_to_int.get(&to).unwrap();
        let key = (from_n, on);
        if let Some(existing) = self.transitions.get(&key) {
            if *existing != to_n {
                panic!("attempting to alter an existing transition");
            }
        } else {
            self.transitions.insert(key, to_n);
        }
    }
}

fn first(grammar: &Grammar, first_and_follow: &FirstAndFollow, symbols: &[Symbol]) -> HashSet<Symbol> {
    let mut result: HashSet<Symbol> = HashSet::new();
    // add the first sets of each individual symbol until a set does not contain epsilon
    for symbol in symbols {
        if let Some(tmp) = first_and_follow.first(symbol) {
            for s in tmp {
                result.insert(*s);
            }
            if !tmp.contains(&grammar.symbol_db().epsilon()) {
                break;
            }
        }
    }
    result.remove(&grammar.symbol_db().epsilon());
    result
}

fn closure(first_and_follow: &FirstAndFollow, grammar: &Grammar, items: BTreeSet<LR1Item>) -> BTreeSet<LR1Item> {
    let mut result = BTreeSet::new();

    // all items in a set are in its closure
    for item in items {
        result.insert(item);
    }

    loop {
        let mut updates: BTreeSet<LR1Item> = BTreeSet::new();
        // for each of the items in the current set of results
        for i in &result {
            // get the sentence after the dot
            let mut unseen = i.symbols_after_dot();
            // if the sentence is not empty and the first symbol is a non-terminal
            if !unseen.is_empty() {
                let s: Symbol = unseen[0];
                if !grammar.symbol_db().is_terminal(&s) {
                    // append the item's lookahead to the sentence
                    unseen.push(*i.lookahead());
                    // and calculate the first of the sentence minus the leading non-terminal
                    let first = first(grammar, first_and_follow, &unseen[1..]);
                    // for every production rule deriving from the non-terminal
                    if let Some(ps) = grammar.productions(&s) {
                        for p in ps {
                            //and every terminal in the previously computed first set
                            for b in &first {
                                // add a new item
                                let prod = LR1Item::new(p.clone(), 0, *b);
                                updates.insert(prod);
                            }
                        }
                    }
                }
            }
        }
        let size_before = result.len();
        // add the updates
        for item in updates {
            result.insert(item);
        }
        let size_after = result.len();
        // stop when no new items are generated
        if size_after == size_before {
            break;
        }
    }

    result
}

fn go_to(first_and_follow: &FirstAndFollow,
         grammar: &Grammar,
         items: &BTreeSet<LR1Item>,
         symbol: &Symbol) -> BTreeSet<LR1Item> {
    let mut result = BTreeSet::new();
    for item in items {
        let unseen = item.symbols_after_dot();
        if !unseen.is_empty() && &unseen[0] == symbol {
            result.insert(LR1Item::new(item.production().clone(), item.dot_position() + 1, item.lookahead().clone()));
        }
    }
    closure(first_and_follow, grammar, result)
}

fn build(grammar: &Grammar) -> CanonicalCollection {
    let symbol_db = grammar.symbol_db();
    let first_and_follow = FirstAndFollow::new(grammar);

    let mut cc = CanonicalCollection {
        next_number: 0,
        int_to_set: BTreeMap::new(),
        set_to_int: BTreeMap::new(),
        transitions: HashMap::new(),
        unprocessed: Vec::new(),
    };

    let p = Production::new(symbol_db.goal(), vec![*grammar.start_symbol()]);
    let mut initial = BTreeSet::new();
    initial.insert(LR1Item::new(p, 0, symbol_db.eoi()));

    let cc0 = closure(&first_and_follow, &grammar, initial);

    cc.add(cc0);

    let mut done = false;
    while !done {
        done = true;
        // for each unprocessed set in cc
        for cc_i in cc.take_unprocessed() {
            // for each item in the set
            for item in &cc_i {
                let unseen = item.symbols_after_dot();
                if !unseen.is_empty() {
                    // if the item is of the form a -> b.xc
                    let x = &unseen[0];
                    // calculate the go_to set for the item and the symbol x
                    let temp = go_to(&first_and_follow, grammar, &cc_i, x);
                    // if this set isn't already part of cc, then add it
                    if !cc.contains(&temp) {
                        cc.add(temp.clone());
                        done = false;
                    }
                    // record the transition from cc_i on the symbol x to the new set
                    cc.add_transition(cc_i.clone(), *x, temp);
                }
            }
        }
    }

    cc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::{Symbol,SymbolDb};

    fn make_item(lhs: Symbol, rhs: Vec<Symbol>, dot: usize, lookahead: Symbol) -> LR1Item {
        LR1Item::new(Production::new(lhs, rhs), dot, lookahead)
    }

    #[test]
    fn closure_01() {
        let mut symbol_db = SymbolDb::new();
        /* grammar:
         *  list -> list pair | pair
         *  pair -> ( pair ) | ( )
         */
        let list = symbol_db.new_nonterminal("list");
        let pair = symbol_db.new_nonterminal("pair");
        let left = symbol_db.new_terminal("(");
        let right = symbol_db.new_terminal(")");
        let goal = symbol_db.goal();
        let eoi = symbol_db.eoi();

        let p1 = Production::new(list, vec![list, pair]);
        let p2 = Production::new(list, vec![pair]);
        let p3 = Production::new(pair, vec![left, pair, right]);
        let p4 = Production::new(pair, vec![left, right]);

        let g = Grammar::new(symbol_db, list, vec![p1, p2, p3, p4]);
        let ff = FirstAndFollow::new(&g);

        let mut closure_items = BTreeSet::new();
        closure_items.insert(make_item(goal, vec![*g.start_symbol()], 0, eoi));
        closure_items.insert(make_item(list, vec![list, pair], 0, eoi));
        closure_items.insert(make_item(list, vec![list, pair], 0, left));
        closure_items.insert(make_item(list, vec![pair], 0, eoi));
        closure_items.insert(make_item(list, vec![pair], 0, left));
        closure_items.insert(make_item(pair, vec![left, pair, right], 0, eoi));
        closure_items.insert(make_item(pair, vec![left, pair, right], 0, left));
        closure_items.insert(make_item(pair, vec![left, right], 0, eoi));
        closure_items.insert(make_item(pair, vec![left, right], 0, left));

        for item in &closure_items {
            let mut s = BTreeSet::new();
            s.insert(item.clone());
            let result = closure(&ff, &g, s);

            for i in &result {
                assert!(closure_items.contains(i))
            }
        }
    }

    #[test]
    fn closure_02() {
        let mut symbol_db = SymbolDb::new();
        /* grammar:
         *  list -> list pair | pair
         *  pair -> ( pair ) | ( )
         */
        let list = symbol_db.new_nonterminal("list");
        let pair = symbol_db.new_nonterminal("pair");
        let left = symbol_db.new_terminal("(");
        let right = symbol_db.new_terminal(")");
        let goal = symbol_db.goal();
        let eoi = symbol_db.eoi();

        let p1 = Production::new(list, vec![list, pair]);
        let p2 = Production::new(list, vec![pair]);
        let p3 = Production::new(pair, vec![left, pair, right]);
        let p4 = Production::new(pair, vec![left, right]);

        let g = Grammar::new(symbol_db, list, vec![p1, p2, p3, p4]);
        let ff = FirstAndFollow::new(&g);

        let mut closure_items = BTreeSet::new();
        closure_items.insert(make_item(goal, vec![list], 1, eoi));
        closure_items.insert(make_item(list, vec![list, pair], 1, eoi));
        closure_items.insert(make_item(list, vec![list, pair], 1, left));
        closure_items.insert(make_item(pair, vec![left, pair, right], 0, eoi));
        closure_items.insert(make_item(pair, vec![left, pair, right], 0, left));
        closure_items.insert(make_item(pair, vec![left, right], 0, eoi));
        closure_items.insert(make_item(pair, vec![left, right], 0, left));

        for item in &closure_items {
            let mut s = BTreeSet::new();
            s.insert(item.clone());
            let result = closure(&ff, &g, s);

            for i in &result {
                assert!(closure_items.contains(i))
            }
        }
    }

    #[test]
    fn go_to_01() {
        let mut symbol_db = SymbolDb::new();
        /* grammar:
         *  list -> list pair | pair
         *  pair -> ( pair ) | ( )
         */
        let list = symbol_db.new_nonterminal("list");
        let pair = symbol_db.new_nonterminal("pair");
        let left = symbol_db.new_terminal("(");
        let right = symbol_db.new_terminal(")");
        let goal = symbol_db.goal();
        let eoi = symbol_db.eoi();

        let p1 = Production::new(list, vec![list, pair]);
        let p2 = Production::new(list, vec![pair]);
        let p3 = Production::new(pair, vec![left, pair, right]);
        let p4 = Production::new(pair, vec![left, right]);

        let g = Grammar::new(symbol_db, list, vec![p1, p2, p3, p4]);
        let ff = FirstAndFollow::new(&g);

        let mut cc_0 = BTreeSet::new();
        cc_0.insert(make_item(goal, vec![*g.start_symbol()], 0, eoi));
        cc_0.insert(make_item(list, vec![list, pair], 0, eoi));
        cc_0.insert(make_item(list, vec![list, pair], 0, left));
        cc_0.insert(make_item(list, vec![pair], 0, eoi));
        cc_0.insert(make_item(list, vec![pair], 0, left));
        cc_0.insert(make_item(pair, vec![left, pair, right], 0, eoi));
        cc_0.insert(make_item(pair, vec![left, pair, right], 0, left));
        cc_0.insert(make_item(pair, vec![left, right], 0, eoi));
        cc_0.insert(make_item(pair, vec![left, right], 0, left));

        let mut cc_1 = BTreeSet::new();
        cc_1.insert(make_item(goal, vec![list], 1, eoi));
        cc_1.insert(make_item(list, vec![list, pair], 1, eoi));
        cc_1.insert(make_item(list, vec![list, pair], 1, left));
        cc_1.insert(make_item(pair, vec![left, pair, right], 0, eoi));
        cc_1.insert(make_item(pair, vec![left, pair, right], 0, left));
        cc_1.insert(make_item(pair, vec![left, right], 0, eoi));
        cc_1.insert(make_item(pair, vec![left, right], 0, left));

        let result = go_to(&ff, &g, &cc_0, &list);
        assert_eq!(result, cc_1);
    }
}


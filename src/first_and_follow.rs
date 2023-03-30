use std::collections::{HashMap,HashSet};

use super::grammar::Grammar;
use super::symbol::Symbol;

#[derive(Debug)]
pub struct FirstAndFollow {
    first: HashMap<Symbol, HashSet<Symbol>>,
    follow: HashMap<Symbol, HashSet<Symbol>>,
}

impl FirstAndFollow {
    pub fn new(grammar: &Grammar) -> FirstAndFollow {
        let first = first(grammar);
        let follow = follow(grammar, &first);
        FirstAndFollow { first , follow }
    }

    pub fn first(&self, s: &Symbol) -> Option<&HashSet<Symbol>> {
        self.first.get(s)
    }

    #[allow(dead_code)]
    pub fn follow(&self, s: &Symbol) -> Option<&HashSet<Symbol>> {
        self.follow.get(s)
    }
}

fn first(grammar: &Grammar) -> HashMap<Symbol,HashSet<Symbol>> {
    let mut first: HashMap<Symbol,HashSet<Symbol>> = HashMap::new();
    
    // for each terminal t, first(t) = {t}
    for s in grammar.terminals() {
        let mut set = HashSet::new();
        set.insert(*s);
        first.insert(*s, set);
    }

    // for each nonterminal nt, initialize first(nt) to an empty set
    for s in grammar.nonterminals() {
        first.insert(*s, HashSet::new());
    }

    let mut done = false;
    while !done {
        done = true;
        // for each of the nonterminals
        for nt in grammar.nonterminals() {
            // iterate through every production
            if let Some(ps) = grammar.productions(nt) {
                for p in ps {
                    let mut new: HashSet<Symbol> = HashSet::new();
                    // for a production A -> a_1 a_2 ... a_n, add first(a_i) to the
                    // set of first items until some first(a_i) does not contain epsilon
                    for a_i in p.rhs() {
                        if let Some(fs) = first.get(a_i) {
                            for s in fs {
                                new.insert(*s);
                            }
                            if !fs.contains(&grammar.symbol_db().epsilon()) {
                                new.remove(&grammar.symbol_db().epsilon());
                                break;
                            }
                        }
                    }
                    // if the computed set contains items that aren't yet in the
                    // first set for this production's LHS, then add those items
                    // and reset the done flag so that the process continues
                    if let Some(fs) = first.get_mut(p.lhs()) {
                        for s in &new {
                            if !fs.contains(s) {
                                fs.insert(*s);
                                done = false;
                            }
                        }
                    }
                }
            }
        }
    }
    
    first
}

fn follow(grammar: &Grammar, first: &HashMap<Symbol,HashSet<Symbol>>) -> HashMap<Symbol,HashSet<Symbol>> {
    let symbol_db = grammar.symbol_db();
    let mut follow: HashMap<Symbol,HashSet<Symbol>> = HashMap::new();
    
    // initialize follow(s) to an empty set for each nonterminal s
    for s in grammar.nonterminals() {
        follow.insert(*s, HashSet::new());
    }

    // add $ to follow(goal)
    let goal = symbol_db.goal();
    let eoi = symbol_db.eoi();
    follow.get_mut(&goal).unwrap().insert(eoi);

    let mut done = false;
    while !done {
        done = true;
        // for each nonterminal nt
        for nt in grammar.nonterminals() {
            // iterate through every production where nt is the lhs
            if let Some(ps) = grammar.productions(nt) {
                for p in ps {
                    // for a production A -> b_1 b_2 ... b_n
                    let mut tail: HashSet<Symbol> = HashSet::new();
                    // set an initial tail set to contain follow(A) as calculated so far
                    if let Some(tmp) = follow.get(nt) {
                        for s in tmp { tail.insert(*s); }
                    }
                    // go through each b_i in reverse order
                    for b_i in p.rhs().iter().rev() {
                        // if b_i is a terminal, then reset tail to first(b_i) which
                        // is just {b_i}
                        if symbol_db.is_terminal(b_i) {
                            tail.clear();
                            tail.insert(*b_i);
                        }
                        // if b_i is a nonterminal
                        else {
                            if let Some(follow_b_i) = follow.get_mut(b_i) {
                                // and tail contains items that are not in follow(b_i)
                                for x in &tail {
                                    if !follow_b_i.contains(x) {
                                        // add the items to follow(b_i)
                                        follow_b_i.insert(*x);
                                        // and indicate that the process must continue
                                        done = false;
                                    }
                                }
                            }
                            // if first(b_i) contains epsilon, then add first(b_i) minus
                            // epsilon to tail. since b_i can derive epsilon, everything in
                            // follow(b_i) will also be in the follow sets of the preceding
                            // b's
                            if let Some(first_b_i) = first.get(b_i) {
                                let epsilon = symbol_db.epsilon();
                                if first_b_i.contains(&epsilon) {
                                    for x in first_b_i {
                                        if x != &epsilon {
                                            tail.insert(*x);
                                        }
                                    }
                                }
                                // if first(b_i) does not contain epsilon, then tail is
                                // reset to contain first(b_i)
                                else {
                                    tail.clear();
                                    for x in first_b_i {
                                        tail.insert(*x);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    follow
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::production::Production;
    use crate::symbol::{SymbolDb};

    /* grammar:
     *   S - a
     */
    #[test]
    fn first_01() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("S");
        let a = symbol_db.new_terminal("a");
        let p1 = Production::new(s, vec![a]);
        let g = Grammar::new(symbol_db, s, vec![p1]);
        let ff = FirstAndFollow::new(&g);
        let first_s = ff.first(&s).unwrap();
        assert_eq!(first_s.len(), 1);
        assert!(first_s.contains(&a));
    }

    /* grammar:
     *   S -> S a | a
     */
    #[test]
    fn first_02() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("S");
        let a = symbol_db.new_terminal("a");
        let p1 = Production::new(s, vec![s, a]);
        let p2 = Production::new(s, vec![a]);
        let g = Grammar::new(symbol_db, s, vec![p1, p2]);
        let ff = FirstAndFollow::new(&g);
        let first_s = ff.first(&s).unwrap();
        assert_eq!(first_s.len(), 1);
        assert!(first_s.contains(&a));
    }

    /* grammar:
     *   S -> S a | a b
     */
    #[test]
    fn first_03() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("S");
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let p1 = Production::new(s, vec![s, a]);
        let p2 = Production::new(s, vec![a, b]);
        let g = Grammar::new(symbol_db, s, vec![p1, p2]);
        let ff = FirstAndFollow::new(&g);
        let first_s = ff.first(&s).unwrap();
        assert_eq!(first_s.len(), 1);
        assert!(first_s.contains(&a));
    }

    /* grammar:
     *   S -> X
     *   X -> a b
     */
    #[test]
    fn first_04() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("S");
        let x = symbol_db.new_nonterminal("X");
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let p1 = Production::new(s, vec![x]);
        let p2 = Production::new(x, vec![a, b]);
        let g = Grammar::new(symbol_db, s, vec![p1, p2]);
        let ff = FirstAndFollow::new(&g);
        let first_s = ff.first(&s).unwrap();
        assert_eq!(first_s.len(), 1);
        assert!(first_s.contains(&a));
        let first_x = ff.first(&x).unwrap();
        assert_eq!(first_x.len(), 1);
        assert!(first_x.contains(&a));
    }

    /* grammar:
     *   S -> X b
     *   X -> a | ε
     */
    #[test]
    fn first_05() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("S");
        let x = symbol_db.new_nonterminal("X");
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let p1 = Production::new(s, vec![x, b]);
        let p2 = Production::new(x, vec![a]);
        let p3 = Production::new(x, vec![symbol_db.epsilon()]);
        let g = Grammar::new(symbol_db, s, vec![p1, p2, p3]);
        let ff = FirstAndFollow::new(&g);
        let first_s = ff.first(&s).unwrap();
        assert_eq!(first_s.len(), 2);
        assert!(first_s.contains(&a));
        assert!(first_s.contains(&b));
        let first_x = ff.first(&x).unwrap();
        assert_eq!(first_x.len(), 2);
        assert!(first_x.contains(&a));
        assert!(first_x.contains(&g.symbol_db().epsilon()));
    }

    /* grammar:
     *   S -> X Y
     *   X -> a | ε
     *   Y -> S | b
     */
    #[test]
    fn first_06() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("S");
        let x = symbol_db.new_nonterminal("X");
        let y = symbol_db.new_nonterminal("Y");
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let p1 = Production::new(s, vec![x, y]);
        let p2 = Production::new(x, vec![a]);
        let p3 = Production::new(x, vec![symbol_db.epsilon()]);
        let p4 = Production::new(y, vec![s]);
        let p5 = Production::new(y, vec![b]);
        let g = Grammar::new(symbol_db, s, vec![p1, p2, p3, p4, p5]);
        let ff = FirstAndFollow::new(&g);
        let first_s = ff.first(&s).unwrap();
        assert_eq!(first_s.len(), 2);
        assert!(first_s.contains(&a));
        assert!(first_s.contains(&b));
        let first_x = ff.first(&x).unwrap();
        assert_eq!(first_x.len(), 2);
        assert!(first_x.contains(&a));
        assert!(first_x.contains(&g.symbol_db().epsilon()));
        let first_y = ff.first(&y).unwrap();
        assert_eq!(first_y.len(), 2);
        assert!(first_y.contains(&a));
        assert!(first_y.contains(&b));
    }

    /* grammar:
     *   expr -> expr + term | expr - term | term
     *   term -> term * factor | term / factor | factor
     *   factor -> ( expr ) | num | name
     */
    #[test]
    fn first_07() {
        let mut symbol_db = SymbolDb::new();
        let expr = symbol_db.new_nonterminal("expr");
        let term = symbol_db.new_nonterminal("term");
        let factor = symbol_db.new_nonterminal("factor");
        let plus = symbol_db.new_terminal("+");
        let minus = symbol_db.new_terminal("-");
        let mult = symbol_db.new_terminal("*");
        let div = symbol_db.new_terminal("/");
        let left = symbol_db.new_terminal("(");
        let right = symbol_db.new_terminal(")");
        let num = symbol_db.new_terminal("num");
        let name = symbol_db.new_terminal("name");
        let p1 = Production::new(expr, vec![expr, plus, term]);
        let p2 = Production::new(expr, vec![expr, minus, term]);
        let p3 = Production::new(expr, vec![term]);
        let p4 = Production::new(term, vec![term, mult, factor]);
        let p5 = Production::new(term, vec![term, div, factor]);
        let p6 = Production::new(term, vec![factor]);
        let p7 = Production::new(factor, vec![left, expr, right]);
        let p8 = Production::new(factor, vec![num]);
        let p9 = Production::new(factor, vec![name]);
        let g = Grammar::new(symbol_db, expr, vec![p1, p2, p3, p4, p5, p6, p7, p8, p9]);
        let ff = FirstAndFollow::new(&g);
        let first_expr = ff.first(&expr).unwrap();
        assert_eq!(first_expr.len(), 3);
        assert!(first_expr.contains(&left));
        assert!(first_expr.contains(&num));
        assert!(first_expr.contains(&name));
        let first_term = ff.first(&expr).unwrap();
        assert_eq!(first_term.len(), 3);
        assert!(first_term.contains(&left));
        assert!(first_term.contains(&num));
        assert!(first_term.contains(&name));
    }

    /* grammar:
     *   expr -> term expr'
     *   expr' -> + term expr' | - term expr' | ε
     *   term -> factor term'
     *   term' -> * factor term' | / factor term' | ε
     */
    #[test]
    fn first_08() {
        let mut symbol_db = SymbolDb::new();
        let expr = symbol_db.new_nonterminal("expr");
        let expr_ = symbol_db.new_nonterminal("expr'");
        let term = symbol_db.new_nonterminal("term");
        let term_ = symbol_db.new_nonterminal("term'");
        let plus = symbol_db.new_terminal("+");
        let minus = symbol_db.new_terminal("-");
        let mult = symbol_db.new_terminal("*");
        let div = symbol_db.new_terminal("/");
        let factor = symbol_db.new_terminal("factor");
        let p1 = Production::new(expr, vec![term, expr_]);
        let p2 = Production::new(expr_, vec![plus, term, expr_]);
        let p3 = Production::new(expr_, vec![minus, term, expr_]);
        let p4 = Production::new(expr_, vec![symbol_db.epsilon()]);
        let p5 = Production::new(term, vec![factor, term_]);
        let p6 = Production::new(term_, vec![mult, factor, term_]);
        let p7 = Production::new(term_, vec![div, factor, term_]);
        let p8 = Production::new(term_, vec![symbol_db.epsilon()]);
        let g = Grammar::new(symbol_db, expr, vec![p1, p2, p3, p4, p5, p6, p7, p8]);
        let ff = FirstAndFollow::new(&g);
        let first_expr = ff.first(&expr).unwrap();
        assert_eq!(first_expr.len(), 1);
        assert!(first_expr.contains(&factor));
        let first_expr_ = ff.first(&expr_).unwrap();
        assert_eq!(first_expr_.len(), 3);
        assert!(first_expr_.contains(&plus));
        assert!(first_expr_.contains(&minus));
        assert!(first_expr_.contains(&g.symbol_db().epsilon()));
        let first_term = ff.first(&term).unwrap();
        assert_eq!(first_term.len(), 1);
        assert!(first_term.contains(&factor));
        let first_term_ = ff.first(&term_).unwrap();
        assert_eq!(first_term_.len(), 3);
        assert!(first_term_.contains(&mult));
        assert!(first_term_.contains(&div));
        assert!(first_term_.contains(&g.symbol_db().epsilon()));
    }

    /* grammar:
     *   S -> X
     *   X -> Y | ε
     *   Y -> Z | ε
     *   Z -> X a | Y b | c
     */
    #[test]
    fn first_09() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("S");
        let x = symbol_db.new_nonterminal("X");
        let y = symbol_db.new_nonterminal("Y");
        let z = symbol_db.new_nonterminal("Z");
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let c = symbol_db.new_terminal("c");
        let p1 = Production::new(s, vec![x]);
        let p2 = Production::new(x, vec![y]);
        let p3 = Production::new(x, vec![symbol_db.epsilon()]);
        let p4 = Production::new(y, vec![z]);
        let p5 = Production::new(y, vec![symbol_db.epsilon()]);
        let p6 = Production::new(z, vec![x, a]);
        let p7 = Production::new(z, vec![y, b]);
        let p8 = Production::new(z, vec![c]);
        let g = Grammar::new(symbol_db, s, vec![p1, p2, p3, p4, p5, p6, p7, p8]);
        let ff = FirstAndFollow::new(&g);
        let first_s = ff.first(&s).unwrap();
        assert_eq!(first_s.len(), 4);
        assert!(first_s.contains(&a));
        assert!(first_s.contains(&b));
        assert!(first_s.contains(&c));
        assert!(first_s.contains(&g.symbol_db().epsilon()));
    }

    /* grammar:
     *   S -> X Y
     *   X -> a | ε
     *   Y -> S | b
     */
    #[test]
    fn follow_01() {
        let mut symbol_db = SymbolDb::new();
        let s = symbol_db.new_nonterminal("S");
        let x = symbol_db.new_nonterminal("X");
        let y = symbol_db.new_nonterminal("Y");
        let a = symbol_db.new_terminal("a");
        let b = symbol_db.new_terminal("b");
        let epsilon = symbol_db.epsilon();
        let eoi = symbol_db.eoi();
        let p1 = Production::new(s, vec![x, y]);
        let p2 = Production::new(x, vec![a]);
        let p3 = Production::new(x, vec![epsilon]);
        let p4 = Production::new(y, vec![s]);
        let p5 = Production::new(y, vec![b]);
        let g = Grammar::new(symbol_db, s, vec![p1, p2, p3, p4, p5]);
        let ff = FirstAndFollow::new(&g);
        {
            let follow = ff.follow(&s).unwrap();
            assert_eq!(follow.len(), 1);
            assert!(follow.contains(&eoi));
        }
        {
            let follow = ff.follow(&x).unwrap();
            assert_eq!(follow.len(), 2);
            assert!(follow.contains(&a));
            assert!(follow.contains(&b));
        }
        {
            let follow = ff.follow(&y).unwrap();
            assert_eq!(follow.len(), 1);
            assert!(follow.contains(&eoi));
        }
    }

    /* grammar:
     *   expr -> term expr'
     *   expr' -> + term expr' | - term expr' | ε
     *   term -> factor term'
     *   term' -> * factor term' | / factor term' | ε
     *   factor -> ( expr ) | num | name
     */
    #[test]
    fn follow_02() {
        let mut symbol_db = SymbolDb::new();
        let expr = symbol_db.new_nonterminal("expr");
        let expr_ = symbol_db.new_nonterminal("expr'");
        let term = symbol_db.new_nonterminal("term");
        let term_ = symbol_db.new_nonterminal("term'");
        let plus = symbol_db.new_terminal("+");
        let minus = symbol_db.new_terminal("-");
        let mult = symbol_db.new_terminal("*");
        let div = symbol_db.new_terminal("/");
        let factor = symbol_db.new_nonterminal("factor");
        let left = symbol_db.new_terminal("(");
        let right = symbol_db.new_terminal(")");
        let num = symbol_db.new_terminal("num");
        let name = symbol_db.new_terminal("name");
        let p1 = Production::new(expr, vec![term, expr_]);
        let p2 = Production::new(expr_, vec![plus, term, expr_]);
        let p3 = Production::new(expr_, vec![minus, term, expr_]);
        let p4 = Production::new(expr_, vec![symbol_db.epsilon()]);
        let p5 = Production::new(term, vec![factor, term_]);
        let p6 = Production::new(term_, vec![mult, factor, term_]);
        let p7 = Production::new(term_, vec![div, factor, term_]);
        let p8 = Production::new(term_, vec![symbol_db.epsilon()]);
        let p9 = Production::new(factor, vec![left, expr, right]);
        let p10 = Production::new(factor, vec![num]);
        let p11 = Production::new(factor, vec![name]);

        let g = Grammar::new(symbol_db, expr, vec![p1, p2, p3, p4, p5, p6, p7, p8, p9, p10, p11]);
        let ff = FirstAndFollow::new(&g);

        let follow_goal = ff.follow(&g.symbol_db().goal()).unwrap();
        assert_eq!(follow_goal.len(), 1);
        assert!(follow_goal.contains(&g.symbol_db().eoi()));

        let follow_expr = ff.follow(&expr).unwrap();
        assert_eq!(follow_expr.len(), 2);
        assert!(follow_expr.contains(&g.symbol_db().eoi()));
        assert!(follow_expr.contains(&right));

        let follow_expr_ = ff.follow(&expr_).unwrap();
        assert_eq!(follow_expr_.len(), 2);
        assert!(follow_expr_.contains(&g.symbol_db().eoi()));
        assert!(follow_expr_.contains(&right));

        let follow_term = ff.follow(&term).unwrap();
        assert_eq!(follow_term.len(), 4);
        assert!(follow_term.contains(&g.symbol_db().eoi()));
        assert!(follow_term.contains(&plus));
        assert!(follow_term.contains(&minus));
        assert!(follow_term.contains(&right));

        let follow_term_ = ff.follow(&term_).unwrap();
        assert_eq!(follow_term_.len(), 4);
        assert!(follow_term_.contains(&g.symbol_db().eoi()));
        assert!(follow_term_.contains(&plus));
        assert!(follow_term_.contains(&minus));
        assert!(follow_term_.contains(&right));

        let follow_factor = ff.follow(&factor).unwrap();
        assert_eq!(follow_factor.len(), 6);
        assert!(follow_factor.contains(&g.symbol_db().eoi()));
        assert!(follow_factor.contains(&plus));
        assert!(follow_factor.contains(&minus));
        assert!(follow_factor.contains(&mult));
        assert!(follow_factor.contains(&div));
        assert!(follow_factor.contains(&right));
    }
}


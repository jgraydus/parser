use super::action::Action;
use super::grammar::Grammar;
use super::parse_tables::ParseTables;
use super::parse_tree::ParseTree;
use super::symbol::Symbol;

pub struct Parser {
    grammar: Grammar,
    parse_tables: ParseTables,
}

impl Parser {
    pub fn new(grammar: Grammar) -> Parser {
        let parse_tables = ParseTables::new(&grammar);
        //println!("{}", parse_tables.to_string(grammar.symbol_db()));
        Parser { grammar, parse_tables }
    }

    pub fn parse<T,F>(&self, tokens: Vec<T>, token_to_symbol: F) -> Option<ParseTree<T>>
        where T: Clone,
              F: Fn(&T) -> Symbol {

        let mut parse_stack: Vec<ParseTree<T>> = Vec::new();
        let mut state_stack: Vec<u32> = Vec::new();

        state_stack.push(0);

        let mut iter = tokens.iter();

        let mut token: &T = iter.next().unwrap();
        let mut symbol: Symbol = token_to_symbol(token);

        loop {
            let state = *state_stack.last().unwrap();

            if let Some(action) = self.parse_tables.action(state, symbol.clone()) {
                //let s = self.grammar.symbol_db().label(&symbol).unwrap();
                //println!("{}, state: {}, action: {}", s, state, action.to_string(self.grammar.symbol_db()));
                match action {
                    Action::Reduce(p) => {
                        let lhs = p.lhs();
                        let rhs: Vec<Symbol> = p.rhs().iter()
                            .cloned()
                            .filter(|s| s != &self.grammar.symbol_db().epsilon())
                            .collect();

                        let size = rhs.len();

                        let mut t = ParseTree::new(lhs.clone(), token.clone());

                        let mut temp = Vec::new();

                        for _ in 0..size {
                            state_stack.pop();
                            temp.push(parse_stack.pop().unwrap());
                        }

                        for _ in 0..size {
                            t.add_child(temp.pop().unwrap());
                        }

                        parse_stack.push(t);
                        let current_state = *state_stack.last().unwrap();
                        if let Some(next_state) = self.parse_tables.transition(current_state, lhs.clone()) {
                            state_stack.push(*next_state);
                        } else {
                            panic!("no entry in transition table for {}", current_state);
                        }
                    },
                    Action::Shift(next_state) => {
                        parse_stack.push(ParseTree::new(symbol.clone(), token.clone()));
                        state_stack.push(*next_state);
                        token = iter.next().unwrap();
                        symbol = token_to_symbol(token);
                    },
                    Action::Accept => {
                        break;
                    }
                }
            } else {
                let s = self.grammar.symbol_db().label(&symbol).unwrap();
                panic!("no entry in action table for ({},{})", state, s);
            }
        }
    
        parse_stack.pop()
    }
}

// cargo test -- --nocapture

#[cfg(test)]
mod tests {
    use super::*;
    use crate::production::Production;
    use crate::symbol::{SymbolDb};

    #[derive(Clone,Debug)]
    enum Token {
        ParenLeft,
        ParenRight,
        Identifier,
        EndOfFile,
    }

    #[test]
    fn test01() {
        let mut symbol_db = SymbolDb::new();
        /* grammar:
         *   e1 -> ( e1 ) | ε
         */
        let e1 = symbol_db.new_nonterminal("E1");
        let lp = symbol_db.new_terminal("(");
        let rp = symbol_db.new_terminal(")");
        let epsilon = symbol_db.epsilon();
        let eoi = symbol_db.eoi();
        let productions = vec![
            Production::new(e1, vec![lp, e1, rp]),
            Production::new(e1, vec![epsilon])
        ];
        let g = Grammar::new(symbol_db, e1, productions);
        use Token::*;
        let ttos = |token: &Token| {
            match token {
                ParenLeft => lp.clone(),
                ParenRight => rp.clone(),
                EndOfFile => eoi,
                _ => eoi,
            }
        };
        let p = Parser::new(g);
        p.parse(vec![ParenLeft, ParenLeft, ParenRight, ParenRight, EndOfFile], ttos);
    }

    #[test]
    fn test02() {
        let mut symbol_db = SymbolDb::new();
        /* grammar:
         *   e1 -> id | e2
         *   e2 -> ( e3 )
         *   e3 -> e1 e3 | ε
         */
        let e1 = symbol_db.new_nonterminal("E1");
        let e2 = symbol_db.new_nonterminal("E2");
        let e3 = symbol_db.new_nonterminal("E3");
        let lp = symbol_db.new_terminal("(");
        let rp = symbol_db.new_terminal(")");
        let id = symbol_db.new_terminal("id");
        let epsilon = symbol_db.epsilon();
        let eoi = symbol_db.eoi();
        let productions = vec![
            Production::new(e1.clone(), vec![id]),
            Production::new(e1.clone(), vec![e2]),
            Production::new(e2.clone(), vec![lp, e3, rp]),
            Production::new(e3.clone(), vec![e1, e3]),
            Production::new(e3.clone(), vec![epsilon]),
        ];
        let g = Grammar::new(symbol_db, e1, productions);

        let ttos = |token: &Token| {
            match token {
                ParenLeft => lp.clone(),
                ParenRight => rp.clone(),
                Identifier => id.clone(),
                EndOfFile => eoi.clone(),
            }
        };

        let p = Parser::new(g);

        use Token::*;

        p.parse(vec![Identifier, EndOfFile], ttos);
        p.parse(vec![ParenLeft, Identifier, ParenRight, EndOfFile], ttos);
        p.parse(vec![ParenLeft, Identifier, Identifier, ParenRight, EndOfFile], ttos);
        p.parse(vec![ParenLeft, Identifier, ParenLeft, Identifier, ParenRight, ParenRight, EndOfFile], ttos);
    }

}


use super::symbol::Symbol;

#[derive(Debug)]
pub struct ParseTree<T> {
    symbol: Symbol,
    token: T,
    children: Vec<ParseTree<T>>
}

impl <T> ParseTree<T> {
    pub fn new(symbol: Symbol, token: T) -> ParseTree<T> {
        ParseTree { symbol, token, children: Vec::new() }
    }

    pub fn token(&self) -> &T {
        &self.token
    }

    pub fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    pub fn children(&self) -> &Vec<ParseTree<T>> {
        &self.children
    }

    pub fn add_child(&mut self, child: ParseTree<T>) {
        self.children.push(child);
    }
}


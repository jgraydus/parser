use super::production::Production;
use super::symbol::{SymbolDb};

#[derive(Debug,Eq,PartialEq)]
pub enum Action {
    Accept,
    Shift(u32),
    Reduce(Production)
}

impl Action {
    pub fn accept() -> Action {
        Action::Accept
    }

    pub fn shift(state: u32) -> Action {
        Action::Shift(state)
    }

    pub fn reduce(p: Production) -> Action {
        Action::Reduce(p)
    }

    #[allow(dead_code)]
    pub fn to_string(&self, symbol_db: &SymbolDb) -> String {
        match self {
            Action::Accept => "Accept".to_string(),
            Action::Shift(n) => format!("Shift({})", n),
            Action::Reduce(p) => format!("Reduce({})", p.to_string(symbol_db)),
        }
    }
}

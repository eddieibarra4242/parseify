use std::collections::HashSet;
use crate::scanner::Token;

#[derive(Debug, Clone)]
pub(crate) struct Production {
  pub(crate) list: Vec<Token>,
  pub(crate) predict_set: HashSet<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct NonTerminal {
  pub(crate) name: String,
  pub(crate) is_start_term: bool,
  pub(crate) is_nullable: bool,
  pub(crate) first_set: HashSet<String>,
  pub(crate) follow_set: HashSet<String>,
  pub(crate) productions: Vec<Production>,
  pub(crate) predict_set: HashSet<String>,
}

impl Production {
  pub(crate) fn new() -> Self {
    Production {
      list: vec![],
      predict_set: HashSet::new(),
    }
  }

  pub(crate) fn push(&mut self, token: Token) {
    self.list.push(token);
  }
}

impl NonTerminal {
  pub(crate) fn new(name: String) -> Self {
    NonTerminal {
      name,
      is_nullable: false,
      is_start_term: false,
      first_set: HashSet::new(),
      follow_set: HashSet::new(),
      productions: vec![],
      predict_set: HashSet::new(),
    }
  }

  pub(crate) fn push(&mut self, production: Production) {
    self.productions.push(production);
  }
}
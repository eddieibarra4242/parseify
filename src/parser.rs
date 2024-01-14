use crate::scanner::Token;

pub(crate) struct Parser {
  scanner: Vec<Token>,
  current_ndx: usize,
}

impl Parser {
  pub(crate) fn new(tokens: Vec<Token>) -> Self {
    Parser {
      scanner: tokens,
      current_ndx: 0,
    }
  }

  fn error(&self, msg: &str, expected: &[&str]) {
    panic!("Parse error {} at {}:  Expected {:?}", msg, self.scanner[self.current_ndx].value, expected);
  }

  fn match_kind(&mut self, kind: &str) -> Token {
    if self.current() == kind {
      let prev = self.scanner[self.current_ndx].clone();
      self.current_ndx += 1;
      return prev;
    } else {
      self.error("", &[kind]);
    }
    self.scanner[self.current_ndx].clone()
  }

  fn current(&self) -> &str {
    self.scanner[self.current_ndx].kind.as_str()
  }

  pub(crate) fn parse(&mut self) {
    self.bnf_file();
    self.match_kind("EOF");
  }

  fn bnf_file(&mut self) {
    if ["ID"].contains(&self.current()) {
      self.production();
      self.production_list();
    } else {
      self.error("syntax error", &["ID"]);
    }
  }

  fn production_list(&mut self) {
    if ["ID"].contains(&self.current()) {
      self.production();
      self.production_list();
    } else if ["EOF"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["EOF", "ID"]);
    }
  }

  fn token_list(&mut self) {
    if ["TERM", "ID"].contains(&self.current()) {
      self.token();
      self.token_list();
    } else if ["END"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["ID", "END", "TERM"]);
    }
  }

  fn production(&mut self) {
    if ["ID"].contains(&self.current()) {
      self.match_kind("ID");
      self.match_kind("EQUALS");
      self.token_list();
      self.match_kind("END");
    } else {
      self.error("syntax error", &["ID"]);
    }
  }

  fn token(&mut self) {
    if ["ID"].contains(&self.current()) {
      self.match_kind("ID");
    } else if ["TERM"].contains(&self.current()) {
      self.match_kind("TERM");
    } else {
      self.error("syntax error", &["TERM", "ID"]);
    }
  }
}
/**
 * Parsify, a simple recursive descent parser generator.
 * Copyright (C) 2024  Eduardo Ibarra
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::HashMap;
use crate::productions::{NonTerminal, Production};
use crate::scanner::Token;

pub(crate) struct Parser {
  scanner: Vec<Token>,
  current_ndx: usize,
  productions: HashMap<String, Vec<Production>>,
  first_nt: String,
}

impl Parser {
  pub(crate) fn new(tokens: Vec<Token>) -> Self {
    Parser {
      scanner: tokens,
      current_ndx: 0,
      productions: HashMap::new(),
      first_nt: String::new(),
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

  pub(crate) fn parse(&mut self) -> Vec<NonTerminal> {
    self.bnf_file();
    self.match_kind("EOF");

    let mut result = vec![];

    for (name, prods) in &self.productions {
      let mut prods_sanitized = vec![];

      for prod in prods {
        let mut new_prod = Production::new();
        for token in &prod.list {
          if token.kind.eq("ID") && !self.productions.contains_key(&token.value) {
            let mut new_token = token.clone();
            new_token.kind = "TERM".to_string();
            new_prod.push(new_token);
          } else {
            new_prod.push(token.clone());
          }
        }

        prods_sanitized.push(new_prod);
      }

      let mut nt = NonTerminal::new(name.clone());
      nt.productions = prods_sanitized;
      nt.is_start_term = self.first_nt.eq(name);
      result.push(nt);
    }

    result
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

  fn token_list(&mut self) -> Production {
    if ["TERM", "ID"].contains(&self.current()) {
      let token = self.token();
      let mut production = self.token_list();

      production.push_to_front(token);
      return production;
    } else if ["END"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["ID", "END", "TERM"]);
    }

    Production::new()
  }

  fn production(&mut self) {
    if ["ID"].contains(&self.current()) {
      let nt = self.match_kind("ID");
      self.match_kind("EQUALS");
      let prod = self.token_list();
      self.match_kind("END");

      if self.first_nt.is_empty() {
        self.first_nt = nt.value.clone();
      }

      if !self.productions.contains_key(&nt.value) {
        self.productions.insert(nt.value.clone(), vec![]);
      }

      self.productions.get_mut(&nt.value).unwrap().push(prod);
    } else {
      self.error("syntax error", &["ID"]);
    }
  }

  fn token(&mut self) -> Token {
    if ["ID"].contains(&self.current()) {
      self.match_kind("ID")
    } else if ["TERM"].contains(&self.current()) {
      self.match_kind("TERM")
    } else {
      self.error("syntax error", &["TERM", "ID"]);
      Token::new()
    }
  }
}
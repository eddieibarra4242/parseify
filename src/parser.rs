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
use crate::parser::ParserError::UnexpectedToken;
use crate::scanner::Token;

#[derive(Debug)]
pub(crate) enum ParserError {
  UnexpectedToken(Token, Vec<&'static str>)
}

pub(crate) struct Parser {
  scanner: Vec<Token>,
  current_ndx: usize,
  productions: HashMap<String, Vec<Production>>,
}

impl Parser {
  pub(crate) fn new(tokens: Vec<Token>) -> Self {
    Parser {
      scanner: tokens,
      current_ndx: 0,
      productions: HashMap::new(),
    }
  }

  fn match_kind(&mut self, kind: &'static str) -> Result<Token, ParserError> {
    return if self.current() == kind {
      let prev = self.scanner[self.current_ndx].clone();
      self.current_ndx += 1;
      Ok(prev)
    } else {
      Err(UnexpectedToken(self.current_token(), vec![kind]))
    };
  }

  fn current(&self) -> &str {
    self.scanner[self.current_ndx].kind.as_str()
  }
  fn current_token(&self) -> Token {
    self.scanner[self.current_ndx].clone()
  }

  pub(crate) fn parse(&mut self) -> Result<Vec<NonTerminal>, ParserError> {
    self.bnf_file()?;
    self.match_kind("EOF")?;

    let mut nt_order: Vec<String> = vec![];

    let mut prev = self.scanner.first().unwrap();
    for token in &self.scanner {
      if token.kind.eq("EQUALS") && !nt_order.contains(&prev.value) {
        nt_order.push(prev.value.clone());
      }

      prev = token;
    }

    let mut result = vec![];

    for name in &nt_order {
      let prods = self.productions.get(name).unwrap();
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
      nt.is_start_term = nt_order.first().unwrap().eq(name);
      result.push(nt);
    }

    Ok(result)
  }

  fn bnf_file(&mut self) -> Result<(), ParserError> {
    if ["ID"].contains(&self.current()) {
      self.production()?;
      self.production_list()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["ID"]));
    }
    Ok(())
  }

  fn production_list(&mut self) -> Result<(), ParserError> {
    if ["ID"].contains(&self.current()) {
      self.production()?;
      self.production_list()?;
    } else if ["EOF"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["EOF", "ID"]));
    }
    Ok(())
  }

  fn production(&mut self) -> Result<(), ParserError> {
    if ["ID"].contains(&self.current()) {
      let nt = self.match_kind("ID")?;
      self.match_kind("EQUALS")?;
      let prod_list = self.rhs()?;
      self.match_kind("END")?;

      if !self.productions.contains_key(&nt.value) {
        self.productions.insert(nt.value.clone(), vec![]);
      }

      self.productions.get_mut(&nt.value).unwrap().extend(prod_list);
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["ID"]))
    }
  }

  fn rhs(&mut self) -> Result<Vec<Production>, ParserError> {
    if ["|", "END", "ID", "TERM"].contains(&self.current()) {
      let prod = self.token_list()?;
      let mut list = self.opt_alternation()?;

      list.insert(0, prod);
      Ok(list)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["|", "END", "ID", "TERM"]))
    }
  }

  fn opt_alternation(&mut self) -> Result<Vec<Production>, ParserError> {
    if ["|"].contains(&self.current()) {
      self.match_kind("|")?;
      let prod = self.token_list()?;
      let mut list = self.opt_alternation()?;

      list.insert(0, prod);
      Ok(list)
    } else if ["END"].contains(&self.current()) {
      // do nothing
      Ok(vec![])
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["|", "END"]))
    }
  }

  fn token_list(&mut self) -> Result<Production, ParserError> {
    if ["ID", "TERM"].contains(&self.current()) {
      let token = self.token()?;
      let mut production = self.token_list()?;

      production.push_to_front(token);
      Ok(production)
    } else if ["|", "END"].contains(&self.current()) {
      // do nothing
      Ok(Production::new())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["|", "END", "ID", "TERM"]))
    }
  }

  fn token(&mut self) -> Result<Token, ParserError> {
    return if ["ID"].contains(&self.current()) {
      Ok(self.match_kind("ID")?)
    } else if ["TERM"].contains(&self.current()) {
      Ok(self.match_kind("TERM")?)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["ID", "TERM"]))
    };
  }
}
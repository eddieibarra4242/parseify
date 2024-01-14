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
  pub(crate) fn push_to_front(&mut self, token: Token) {
    self.list.insert(0, token);
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
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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Wrapper {
  pub(crate) prefix: String,
  pub(crate) suffix: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ReqFunctions {
  pub(crate) constructor: Vec<String>,
  pub(crate) error_func: Vec<String>,
  pub(crate) match_func: Vec<String>,
  pub(crate) current_func: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Language {
  pub(crate) imports: String,
  pub(crate) parse_error: String,
  pub(crate) class_def: String,
  pub(crate) class_body_wrapper: Wrapper,
  pub(crate) required_functions: ReqFunctions,
  pub(crate) func_call: Wrapper,
  pub(crate) match_call: Wrapper,
  pub(crate) error_call: Wrapper,
  pub(crate) condition: Wrapper,
  pub(crate) if_clause: Wrapper,
  pub(crate) elseif_clause: Wrapper,
  pub(crate) else_clause: String,
  pub(crate) public_func_def: Wrapper,
  pub(crate) private_func_def: Wrapper,
  pub(crate) func_body: Wrapper,
  pub(crate) empty_production_body: String,
}

impl Wrapper {
  pub(crate) fn wrap(&self, content: &str) -> String {
    let mut result = self.prefix.clone();
    result.push_str(content);
    result.push_str(self.suffix.as_str());
    result
  }
}
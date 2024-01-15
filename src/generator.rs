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

use std::collections::BTreeSet;
use crate::language::Language;
use crate::productions::{NonTerminal, Production};

const TAB_WIDTH: usize = 4;

struct GeneratorContext {
  num_tabs: usize,
  output: String,
}

impl GeneratorContext {
  fn new() -> Self {
    GeneratorContext {
      num_tabs: 0,
      output: String::new(),
    }
  }

  fn push_tabs(&mut self) {
    self.num_tabs += 1;
  }

  fn pop_tabs(&mut self) {
    if self.num_tabs != 0 {
      self.num_tabs -= 1;
    }
  }

  fn start_line(&mut self) {
    for _ in 0..self.num_tabs {
      for __ in 0..TAB_WIDTH {
        self.output.push(' ');
      }
    }
  }

  fn emit_newline(&mut self) {
    self.output.push('\n');
  }

  fn push_str(&mut self, value: &str) {
    self.output.push_str(value);
  }
}

fn strip_literal(literal: &String) -> String {
  literal.replace("'", "").replace("\"", "")
}

fn normalize_literal(literal: &String) -> String {
  let mut result = String::from('"');
  result.push_str(strip_literal(literal).as_str());
  result.push('"');
  result
}

pub(crate) fn generate_parser(non_terminals: &Vec<NonTerminal>, language: &Language) -> String {
  let mut result = language.imports.clone();
  result.push('\n');
  result.push_str(language.parse_error.as_str());
  result.push('\n');
  result.push_str(language.class_def.as_str());
  result.push('\n');
  result.push_str(generate_class_body(non_terminals, language).as_str());
  result
}

fn generate_class_body(non_terminals: &Vec<NonTerminal>, language: &Language) -> String {
  let mut ctx = GeneratorContext::new();
  ctx.push_tabs();

  let start_term = non_terminals.iter().find(|x| { x.is_start_term }).unwrap();

  emit_required_functions(&mut ctx, language, start_term.name.as_str());

  for nt in non_terminals {
    emit_nonterminal_function(&mut ctx, nt, language);
  }

  ctx.pop_tabs();
  language.class_body_wrapper.wrap(ctx.output.as_str())
}

fn emit_required_functions(ctx: &mut GeneratorContext, language: &Language, start_term_name: &str) {
  for line in &language.required_functions.constructor {
    ctx.start_line();
    ctx.push_str(line.as_str());
    ctx.emit_newline();
  }

  ctx.emit_newline();

  for line in &language.required_functions.error_func {
    ctx.start_line();
    ctx.push_str(line.as_str());
    ctx.emit_newline();
  }

  ctx.emit_newline();

  for line in &language.required_functions.match_func {
    ctx.start_line();
    ctx.push_str(line.as_str());
    ctx.emit_newline();
  }

  ctx.emit_newline();

  for line in &language.required_functions.current_func {
    ctx.start_line();
    ctx.push_str(line.as_str());
    ctx.emit_newline();
  }

  ctx.emit_newline();
  ctx.start_line();
  ctx.push_str(language.public_func_def.wrap("parse").as_str());

  // manually wrap function body to keep tab information.
  // Todo: try a stack based wrapper applier in GeneratorContext (FUNC_WRAPPER_NOTE)
  ctx.push_str(language.func_body.prefix.as_str());
  ctx.emit_newline();
  ctx.push_tabs();

  ctx.start_line();
  ctx.push_str(language.func_call.wrap(start_term_name).as_str());
  ctx.emit_newline();

  ctx.start_line();
  ctx.push_str(language.match_call.wrap("EOF").as_str());
  ctx.emit_newline();

  ctx.pop_tabs();
  ctx.emit_newline();

  ctx.start_line();
  ctx.push_str(language.func_body.suffix.as_str());
  ctx.emit_newline();
}

fn emit_nonterminal_function(ctx: &mut GeneratorContext, nt: &NonTerminal, language: &Language) {
  ctx.start_line();
  ctx.push_str(language.private_func_def.wrap(nt.name.as_str()).as_str());

  // See FUNC_WRAPPER_NOTE
  ctx.push_str(language.func_body.prefix.as_str());
  ctx.emit_newline();
  ctx.push_tabs();
  emit_nonterminal_function_body(ctx, nt, language);
  ctx.pop_tabs();
  ctx.start_line();
  ctx.push_str(language.func_body.suffix.as_str());
  ctx.emit_newline();
}

fn emit_nonterminal_function_body(ctx: &mut GeneratorContext, nt: &NonTerminal, language: &Language) {
  let mut first_prod = true;

  for prod in &nt.productions {
    let wrapper =
      if first_prod {
        first_prod = false;
        ctx.start_line();
        &language.if_clause
      } else {
        &language.elseif_clause
      };

    ctx.push_str(wrapper.wrap(language.condition.wrap(generate_predict_list(&prod.predict_set).as_str()).as_str()).as_str());

    // See FUNC_WRAPPER_NOTE
    ctx.push_str(language.func_body.prefix.as_str());
    ctx.emit_newline();
    ctx.push_tabs();
    emit_production_body(ctx, prod, language);
    ctx.pop_tabs();
    ctx.start_line();
    ctx.push_str(language.func_body.suffix.as_str());
  }

  ctx.push_str(language.else_clause.as_str());

  // See FUNC_WRAPPER_NOTE
  ctx.push_str(language.func_body.prefix.as_str());
  ctx.emit_newline();

  ctx.push_tabs();
  ctx.start_line();
  ctx.push_str(language.error_call.wrap(generate_predict_list(&nt.predict_set).as_str()).as_str());
  ctx.emit_newline();
  ctx.pop_tabs();
  ctx.start_line();
  ctx.push_str(language.func_body.suffix.as_str());
  ctx.emit_newline();
}

fn generate_predict_list(predict_set: &BTreeSet<String>) -> String {
  let mut predict_list = String::new();

  for token in predict_set {
    if token.is_empty() {
      predict_list.push_str("\"EOF\"");
    } else {
      predict_list.push_str(normalize_literal(token).as_str());
    }

    predict_list.push_str(", ");
  }

  predict_list.pop();
  predict_list.pop();
  predict_list
}

fn emit_production_body(ctx: &mut GeneratorContext, prod: &Production, language: &Language) {
  if prod.list.is_empty() {
    ctx.start_line();
    ctx.push_str(language.empty_production_body.as_str());
    ctx.emit_newline();
    return;
  }

  for token in &prod.list {
    ctx.start_line();
    let content = match token.kind.as_str() {
      "TERM" => language.match_call.wrap(strip_literal(&token.value).as_str()),
      "ID" => language.func_call.wrap(token.value.as_str()),
      "EOF" => language.match_call.wrap("EOF"),
      _ => { "".to_string() }
    };

    ctx.push_str(content.as_str());
    ctx.emit_newline();
  }
}

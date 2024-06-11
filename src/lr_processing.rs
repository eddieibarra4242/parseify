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

use crate::productions::{NonTerminal, Production};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use crate::error_handler::resolve_actions_to_string;
use crate::lr_processing::Action::{Accept, Reduce, Shift};
use crate::scanner::{Coord, Span, Token};

#[derive(Clone)]
pub(crate) enum Action {
  Accept,
  Shift(u64),
  Reduce(Vec<Token>, String)
}

#[derive(Clone, Eq, PartialEq)]
struct ContextualProduction {
  nt_name: String,
  matched: Vec<Token>,
  will_match: Vec<Token>,
  predict_set: BTreeSet<String>
}

#[derive(Clone, Eq, PartialEq)]
struct Closure {
  prods: Vec<ContextualProduction>,
  transitions: BTreeMap<String, Box<Closure>>
}

pub(crate) struct State {
  // Vec<Action> allows shift-reduce and reduce-reduce ambiguity
  pub(crate) common_actions: Vec<Action>,
  pub(crate) actions: HashMap<String, Vec<Action>>,
  pub(crate) nt_state_transitions: HashMap<String, u64>
}

pub(crate) struct StateTable {
  pub(crate) states: Vec<State>,
  pub(crate) seen_terms: BTreeSet<String>,
  pub(crate) seen_non_terms: BTreeSet<String>
}

impl ContextualProduction {
  fn new(nt_name: String, production: &Production, predict_set: BTreeSet<String>) -> Self {
    let will_match = production.list.clone();
    ContextualProduction {
      nt_name,
      matched: vec![],
      will_match,
      predict_set,
    }
  }

  fn new_advanced(prev: &ContextualProduction) -> Self {
    let mut matched = prev.matched.clone();
    let mut will_match = prev.will_match.clone();
    matched.push(will_match.remove(0));

    ContextualProduction {
      nt_name: prev.nt_name.clone(),
      matched,
      will_match,
      predict_set: prev.predict_set.clone(),
    }
  }
}

impl Closure {
  fn new() -> Self {
    Closure {
      prods: vec![],
      transitions: BTreeMap::new(),
    }
  }
}

impl State {
  fn new() -> Self {
    State {
      common_actions: vec![],
      actions: HashMap::new(),
      nt_state_transitions: HashMap::new(),
    }
  }
}

impl StateTable {
  fn new() -> Self {
    StateTable {
      states: vec![],
      seen_terms: BTreeSet::new(),
      seen_non_terms: BTreeSet::new(),
    }
  }
}

fn can_append_to_production_set(set: &Vec<ContextualProduction>, appendage: &ContextualProduction) -> bool {
  for prod in set {
    if prod == appendage {
      return false;
    }
  }

  true
}

fn can_append_to_set(set: &Vec<Closure>, appendage: &Closure) -> bool {
  for clo in set {
    if clo.prods == appendage.prods {
      return false;
    }
  }

  true
}

fn find_closure_index(closure_set: &Vec<Closure>, closure: &Closure) -> Option<u64> {
  for i in 0..closure_set.len() {
    let clo = &closure_set[i];
    if clo.prods.eq(&closure.prods) {
      return Some(i as u64);
    }
  }

  None
}

pub(crate) fn lr_process(non_terminals: &Vec<NonTerminal>, is_k0: bool) -> StateTable {
  let mut nt_lookup = HashMap::new();
  let mut start_nt_name: String = String::new();
  for nt in non_terminals {
    if nt.is_start_term {
      start_nt_name = nt.name.clone();
    }

    nt_lookup.insert(nt.name.clone(), nt);
  }

  let mut inner_prod = Production::new();
  inner_prod.push(Token {
    kind: "".to_string(),
    value: start_nt_name.clone(),
    span: Span { start: Coord { line_num: 0, col: 0 }, end: Coord { line_num: 0, col: 0 } },
  });

  let mut start_predict_set = BTreeSet::new();
  start_predict_set.insert("EOF".to_string());
  let start_prod = ContextualProduction::new(String::new(), &inner_prod, start_predict_set);

  let mut closure_set = vec![];
  let mut root_closure = Closure::new();
  root_closure.prods.push(start_prod);

  fill_out_automaton(&nt_lookup, &mut root_closure, &mut closure_set, is_k0);
  closure_set.clear();
  recalculate_set(&root_closure, &mut closure_set);

  let mut state_table = StateTable::new();
  for current in &closure_set {
    let mut state = State::new();

    for prod in &current.prods {
      if !prod.will_match.is_empty() {
        continue;
      }

      if is_k0 {
        if prod.nt_name.is_empty() {
          let eof_str = "EOF".to_string();
          state_table.seen_terms.insert(eof_str.clone());
          if !state.actions.contains_key(&eof_str) {
            state.actions.insert(eof_str.clone(), vec![]);
          }

          state.actions.get_mut(&eof_str).unwrap().push(Accept);
          continue;
        }

        state.common_actions.push(Reduce(prod.matched.clone(), prod.nt_name.clone()));
        continue;
      }

      for predictor in &prod.predict_set {
        state_table.seen_terms.insert(predictor.clone());

        if !state.actions.contains_key(predictor) {
          state.actions.insert(predictor.clone(), vec![]);
        }

        if prod.nt_name.is_empty() {
          state.actions.get_mut(predictor).unwrap().push(Accept);
          continue;
        }

        state.actions.get_mut(predictor).unwrap().push(Reduce(prod.matched.clone(), prod.nt_name.clone()));
      }
    }

    for (transition_value, next_closure) in &current.transitions {
      let next_state_index: Option<u64> = find_closure_index(&closure_set, next_closure);

      if next_state_index.is_none() {
        panic!("Unable to find State!");
      }

      if nt_lookup.contains_key(transition_value) {
        state_table.seen_non_terms.insert(transition_value.clone());
        state.nt_state_transitions.insert(transition_value.clone(), next_state_index.unwrap());
      } else {
        state_table.seen_terms.insert(transition_value.clone());

        if !state.actions.contains_key(transition_value) {
          state.actions.insert(transition_value.clone(), vec![]);
        }

        state.actions.get_mut(transition_value).unwrap().push(Shift(next_state_index.unwrap()));
      }
    }

    state_table.states.push(state);
  }

  check_ambiguities(&state_table);

  state_table
}

fn fill_out_automaton(nt_lookup: &HashMap<String, &NonTerminal>, root: &mut Closure, closure_set: &mut Vec<Closure>, is_k0: bool) {
  closure(nt_lookup, root, is_k0);

  if !can_append_to_set(closure_set, root) {
    return;
  }

  goto(root);
  closure_set.push(root.clone());

  for value in root.transitions.values_mut() {
    fill_out_automaton(nt_lookup, value, closure_set, is_k0);
  }
}

fn recalculate_set(root: &Closure, closure_set: &mut Vec<Closure>) {
  if !can_append_to_set(closure_set, root) {
    return;
  }

  closure_set.push(root.clone());

  for value in root.transitions.values() {
    recalculate_set(value, closure_set);
  }
}

fn calculate_followers(nt_lookup: &HashMap<String, &NonTerminal>, parent_prod: &ContextualProduction) -> BTreeSet<String> {
  let mut followers = BTreeSet::new();
  let mut append_parent_set = true;

  for i in 1..parent_prod.will_match.len() {
    let token = &parent_prod.will_match[i];

    if !nt_lookup.contains_key(&token.value) {
      followers.insert(token.value.clone());
      append_parent_set = false;
      break;
    }

    let nt = nt_lookup.get(&token.value).unwrap();
    followers.extend(nt.first_set.clone());

    if !nt.is_nullable {
      append_parent_set = false;
      break;
    }
  }

  if append_parent_set {
    followers.extend(parent_prod.predict_set.clone());
  }

  followers
}

fn production_closure(nt_lookup: &HashMap<String, &NonTerminal>, seen_nts: &mut HashSet<String>, production: &ContextualProduction, production_set: &mut Vec<ContextualProduction>, is_k0: bool) {
  if !can_append_to_production_set(production_set, production) {
    return;
  }

  production_set.push(production.clone());
  if production.will_match.is_empty() || !nt_lookup.contains_key(&production.will_match.first().unwrap().value) {
    return;
  }

  let nt = nt_lookup.get(&production.will_match.first().unwrap().value).unwrap();
  let followers =
    if is_k0 {
      BTreeSet::new()
    } else {
      calculate_followers(nt_lookup, production)
    };

  if seen_nts.contains(&nt.name) {
    for prod in production_set {
      if prod.nt_name == nt.name {
        prod.predict_set.extend(followers.clone());
      }
    }

    return;
  }

  seen_nts.insert(nt.name.clone());
  for prod in &nt.productions {
    let context_prod = ContextualProduction::new(nt.name.clone(), prod, followers.clone());
    production_closure(nt_lookup, seen_nts, &context_prod, production_set, is_k0);
  }
}

fn closure(nt_lookup: &HashMap<String, &NonTerminal>, closure_so_far: &mut Closure, is_k0: bool) {
  let mut seen_nts = HashSet::new();
  let initial_productions = closure_so_far.prods.clone();
  closure_so_far.prods.clear();
  for prod in initial_productions {
    production_closure(nt_lookup, &mut seen_nts, &prod, &mut closure_so_far.prods, is_k0);
  }
}

fn goto(closure: &mut Closure) {
  for prod in &closure.prods {
    if prod.will_match.is_empty() {
      continue;
    }

    let transition_value = prod.will_match.first().unwrap();
    if !closure.transitions.contains_key(&transition_value.value) {
      closure.transitions.insert(transition_value.value.clone(), Box::new(Closure::new()));
    }

    let advanced_prod = ContextualProduction::new_advanced(&prod);
    closure.transitions.get_mut(&transition_value.value).unwrap().as_mut().prods.push(advanced_prod);
  }
}

fn check_ambiguities(state_table: &StateTable) {
  for i in 0..state_table.states.len() {
    let state = &state_table.states[i];

    for (terminal, actions) in &state.actions {
      if (actions.len() + state.common_actions.len()) <= 1 {
        continue;
      }

      println!("State {} contains an ambiguity for lookahead {}:", i, terminal);
      println!("  Actions: {}\n", resolve_actions_to_string(actions, &state.common_actions));
    }
  }
}
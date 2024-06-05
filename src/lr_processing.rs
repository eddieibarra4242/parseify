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
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use crate::lr_processing::Action::{Accept, Reduce, Shift};
use crate::scanner::{Coord, Span, Token};

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
  seen_nts: HashSet<String>,
  prods: Vec<ContextualProduction>,
  transitions: HashMap<String, Box<Closure>>
}

pub(crate) struct State {
  pub(crate) index: u64,

  // Vec<Action> allows shift-reduce and reduce-reduce ambiguity
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
      seen_nts: HashSet::new(),
      prods: vec![],
      transitions: HashMap::new(),
    }
  }
}

impl State {
  fn new(index: u64) -> Self {
    State {
      index,
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

fn can_append_to_set(set: &Vec<Closure>, appendage: &Closure) -> bool {
  for clo in set {
    if clo.eq(appendage) {
      return false;
    }
  }

  true
}

fn find_closure_index(closure_set: &Vec<Closure>, closure: &Closure) -> Option<u64> {
  for i in 0..closure_set.len() {
    let clo = &closure_set[i];
    if clo.seen_nts.eq(&closure.seen_nts) && clo.prods.eq(&closure.prods) {
      return Some(i as u64);
    }
  }

  None
}

pub(crate) fn lr_process(non_terminals: &Vec<NonTerminal>) -> StateTable {
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

  let mut closure_queue = VecDeque::new();
  closure_queue.push_back(&mut root_closure);

  while !closure_queue.is_empty() {
    let current = closure_queue.pop_front().unwrap();
    closure(&nt_lookup, current);
    goto(current);

    if can_append_to_set(&closure_set, current) {
      closure_set.push(current.clone());
    }

    for value in current.transitions.values_mut() {
      closure_queue.push_back(value)
    }
  }

  let mut state_table = StateTable::new();
  let mut index = 0u64;
  for current in &closure_set {
    let mut state = State::new(index);
    index += 1;

    for prod in &current.prods {
      if !prod.will_match.is_empty() {
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
      // FIXME: there are different Closures in the tree and in the closure set. The Closures in the tree are not updated, so we need to recalculate the closure here, which is unnecessary.
      let mut checkers = next_closure.as_ref().clone();
      closure(&nt_lookup, &mut checkers);

      let next_state_index: Option<u64> = find_closure_index(&closure_set, &checkers);

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

  state_table
}

fn closure(nt_lookup: &HashMap<String, &NonTerminal>, closure_so_far: &mut Closure) {
  let mut closure_queue = VecDeque::new();
  for prod in &closure_so_far.prods {
    closure_queue.push_back(prod.clone());
  }

  while !closure_queue.is_empty() {
    let current = closure_queue.pop_front().unwrap();

    if current.will_match.is_empty() || !nt_lookup.contains_key(&current.will_match.first().unwrap().value) {
      continue;
    }

    let nt = nt_lookup.get(&current.will_match.first().unwrap().value).unwrap();

    if closure_so_far.seen_nts.contains(&nt.name) {
      continue;
    }

    closure_so_far.seen_nts.insert(nt.name.clone());
    for prod in &nt.productions {
      let context_prod = ContextualProduction::new(nt.name.clone(), prod, nt.follow_set.clone());
      closure_so_far.prods.push(context_prod.clone());
      closure_queue.push_back(context_prod);
    }
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
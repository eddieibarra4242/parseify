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

use std::collections::{HashMap, BTreeSet, HashSet};
use std::hash::{Hash, Hasher};
use crate::error_handler::print_ambiguity;
use crate::productions::Nullable::{Maybe, No, Yes};
use crate::scanner::Token;

#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) enum Nullable {
  No,
  Maybe,
  Yes
}

#[derive(Debug, Clone)]
pub(crate) struct Production {
  pub(crate) list: Vec<Token>,
  pub(crate) predict_set: BTreeSet<String>,
  nullable: Nullable,
}

#[derive(Debug, Clone)]
pub(crate) struct NonTerminal {
  pub(crate) name: String,
  pub(crate) is_start_term: bool,
  pub(crate) is_nullable: bool,
  pub(crate) first_set: BTreeSet<String>,
  pub(crate) follow_set: BTreeSet<String>,
  pub(crate) productions: Vec<Production>,
  pub(crate) predict_set: BTreeSet<String>,
}

impl Production {
  pub(crate) fn new() -> Self {
    Production {
      list: vec![],
      predict_set: BTreeSet::new(),
      nullable: Maybe
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
      first_set: BTreeSet::new(),
      follow_set: BTreeSet::new(),
      productions: vec![],
      predict_set: BTreeSet::new(),
    }
  }
}


#[derive(Eq, Debug, Clone)]
struct Node {
  key: String,
  is_term: bool,
}

impl Node {
  fn new(name: String, is_term: bool) -> Self {
    Node { key: name, is_term }
  }
}

impl PartialEq for Node {
  fn eq(&self, other: &Self) -> bool {
    other.key.eq(&self.key)
  }
}

impl Hash for Node {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.key.hash(state);
  }
}

fn first_n_follow_set_dfs(
  first_set: &mut BTreeSet<String>,
  graph: &HashMap<String, HashSet<Node>>,
  start: String,
  visited: &mut Vec<String>,
) {
  visited.push(start.clone());

  for adj in graph.get(&start).unwrap() {
    if visited.contains(&adj.key) {
      continue;
    }

    if adj.is_term {
      first_set.insert(adj.key.clone());
    }

    first_n_follow_set_dfs(first_set, graph, adj.key.clone(), visited);
  }
}

pub(crate) fn process(non_terminals: &mut Vec<NonTerminal>) {
  nullability(non_terminals);

  let mut nullable_info: HashMap<String, bool> = HashMap::new();
  for nt_inner in &mut *non_terminals {
    nullable_info.insert(nt_inner.name.clone(), nt_inner.is_nullable);
  }

  first_sets(non_terminals, &nullable_info);
  follow_sets(non_terminals, &nullable_info);
  predict_sets(non_terminals);
  find_ambiguities(non_terminals);

  for nt in &mut *non_terminals {
    for prod in &mut nt.productions {
      for x in &prod.predict_set {
        nt.predict_set.insert(x.clone());
      }

      if prod.list.is_empty() {
        continue;
      }

      if prod.list.last().unwrap().kind == "EOF" {
        prod.list.pop();
      }
    }
  }
}

pub(crate) fn nullability(nts: &mut Vec<NonTerminal>) {
  let mut nt_nullable_info: HashMap<String, Nullable> = HashMap::new();

  for nt in &mut *nts {
    nt_nullable_info.insert(nt.name.clone(), Maybe);
  }

  let mut should_recompute = true;
  while should_recompute {
    should_recompute = false;

    for nt in &mut *nts {
      if !nt_nullable_info[&nt.name].eq(&Maybe) {
        continue;
      }

      for prod in &mut nt.productions {
        if prod.list.is_empty() {
          prod.nullable = Yes;
          should_recompute = true;
        }

        if prod.nullable.eq(&Yes) {
          nt_nullable_info.insert(nt.name.clone(), Yes);
          should_recompute = true;
          break;
        }

        if prod.nullable.eq(&No) {
          continue;
        }

        let mut is_definitely_null = true;
        for token in &prod.list {
          if token.kind.eq("TERM") {
            prod.nullable = No;
            is_definitely_null = false;
            should_recompute = true;
            break;
          }

          if !nt_nullable_info[&token.value].eq(&Yes) {
            is_definitely_null = false;
          }
        }

        if is_definitely_null {
          prod.nullable = Yes;
          should_recompute = true;
        }
      }
    }
  }

  for nt in &mut *nts {
    nt.is_nullable = nt_nullable_info[&nt.name].eq(&Yes);
  }
}

pub(crate) fn first_sets(nts: &mut Vec<NonTerminal>, nullable_info: &HashMap<String, bool>) {
  let mut graph: HashMap<String, HashSet<Node>> = HashMap::new();

  for nt in &mut *nts {
    if !graph.contains_key(&nt.name) {
      graph.insert(nt.name.clone(), HashSet::new());
    }

    for prod in &nt.productions {
      for token in &prod.list {
        if !graph.contains_key(&token.value) {
          graph.insert(token.value.clone(), HashSet::new());
        }

        let is_nt = token.kind.eq("ID");
        let new_node = Node::new(token.value.clone(), !is_nt);
        graph.get_mut(&nt.name).as_mut().unwrap().insert(new_node);

        if !is_nt || !(*nullable_info.get(&token.value).unwrap()) {
          break;
        }
      }
    }
  }

  for nt in nts {
    let mut visited: Vec<String> = vec![];
    first_n_follow_set_dfs(&mut nt.first_set, &graph, nt.name.clone(), &mut visited);
  }
}

pub(crate) fn follow_sets(nts: &mut Vec<NonTerminal>, nullable_info: &HashMap<String, bool>) {
  let mut graph: HashMap<String, HashSet<Node>> = HashMap::new();
  graph.insert("EOF".to_string(), HashSet::new()); // insert EOF into the graph.

  let nts_clone = nts.clone();

  for nt in &mut *nts {
    if !graph.contains_key(&nt.name) {
      graph.insert(nt.name.clone(), HashSet::new());
    }

    for prod in &nt.productions {
      if prod.list.is_empty() {
        continue;
      }

      let mut previous_token = &prod.list[0];
      if !graph.contains_key(&previous_token.value) {
        graph.insert(previous_token.value.clone(), HashSet::new());
      }

      for i in 1..prod.list.len() {
        let token = &prod.list[i];
        if !graph.contains_key(&token.value) {
          graph.insert(token.value.clone(), HashSet::new());
        }

        if !previous_token.kind.eq("ID") {
          previous_token = token;
          continue;
        }

        let is_nt = token.kind.eq("ID");
        if is_nt {
          let value = nts_clone.iter().find(|x| x.name.eq(&token.value)).unwrap();
          for term in &value.first_set {
            let new_node = Node::new(term.clone(), true);
            graph
              .get_mut(&previous_token.value)
              .as_mut()
              .unwrap()
              .insert(new_node);
          }
        } else {
          let new_node = Node::new(token.value.clone(), true);
          graph
            .get_mut(&previous_token.value)
            .as_mut()
            .unwrap()
            .insert(new_node);
        }

        if *nullable_info.get(&token.value).unwrap_or(&false) {
          // Todo: iterate through list until something not nullable is found. Once it is finish.
          let mut found_nonnullable = false;
          for j in (i + 1)..prod.list.len() {
            let j_token = &prod.list[j];
            if !graph.contains_key(&j_token.value) {
              graph.insert(j_token.value.clone(), HashSet::new());
            }

            if j_token.kind.eq("ID") {
              let value = nts_clone.iter().find(|x| x.name.eq(&j_token.value)).unwrap();
              for term in &value.first_set {
                let new_node = Node::new(term.clone(), true);
                graph
                  .get_mut(&previous_token.value)
                  .as_mut()
                  .unwrap()
                  .insert(new_node);
              }
            } else {
              let new_node = Node::new(j_token.value.clone(), true);
              graph
                .get_mut(&previous_token.value)
                .as_mut()
                .unwrap()
                .insert(new_node);
            }

            if !(*nullable_info.get(&j_token.value).unwrap_or(&false)) {
              found_nonnullable = true;
              break;
            }
          }

          if !found_nonnullable {
            let new_node = Node::new(nt.name.clone(), false);
            graph
              .get_mut(&previous_token.value)
              .as_mut()
              .unwrap()
              .insert(new_node);
          }
        }

        previous_token = token;
      }

      if previous_token.kind.eq("ID") {
        let new_node = Node::new(nt.name.clone(), false);
        graph
          .get_mut(&previous_token.value)
          .as_mut()
          .unwrap()
          .insert(new_node);
      }
    }

    if nt.is_start_term {
      let new_node = Node::new("EOF".to_string(), true);
      graph
        .get_mut(&nt.name)
        .as_mut()
        .unwrap()
        .insert(new_node);
    }
  }

  for nt in nts {
    let mut visited: Vec<String> = vec![];
    first_n_follow_set_dfs(&mut nt.follow_set, &graph, nt.name.clone(), &mut visited);
  }
}

pub(crate) fn predict_sets(nts: &mut Vec<NonTerminal>) {
  let nts_clone = nts.clone();
  for nt in &mut *nts {
    for prod in &mut nt.productions {
      let mut should_add_follow = true;
      for token in &prod.list {
        if !token.kind.eq("ID") {
          prod.predict_set.insert(token.value.clone());
          should_add_follow = false;
          break;
        }

        let token_as_nt = nts_clone.iter().find(|x| x.name.eq(&token.value)).unwrap();
        token_as_nt.first_set.iter().for_each(|x| {
          prod.predict_set.insert(x.clone());
        });

        if !token_as_nt.is_nullable {
          should_add_follow = false;
          break;
        }
      }

      if should_add_follow {
        nt.follow_set.iter().for_each(|x| {
          prod.predict_set.insert(x.clone());
        });
      }
    }
  }
}

pub(crate) fn find_ambiguities(nts: &Vec<NonTerminal>) {
  for nt in nts {
    let mut seen_prediction_tokens = BTreeSet::new();

    for prod in &nt.productions {
      let intersection = seen_prediction_tokens.intersection(&prod.predict_set);
      if intersection.clone().count() > 0 {
        print_ambiguity(&nt.name, intersection);
      }

      let mut vector = vec![];
      for str in &prod.predict_set {
        vector.push(str.clone());
      }

      seen_prediction_tokens.extend(vector);
    }
  }
}

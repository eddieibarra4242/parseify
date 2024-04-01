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
use crate::scanner::Token;

#[derive(Debug, Clone)]
pub(crate) struct Production {
  pub(crate) list: Vec<Token>,
  pub(crate) predict_set: BTreeSet<String>,
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

fn nullable_dfs(
  nullable_info: &mut HashMap<String, bool>,
  graph: &HashMap<String, HashSet<Node>>,
  start: String,
  visited: &mut Vec<String>,
) {
  visited.push(start.clone());

  if nullable_info.contains_key(&start) && *nullable_info.get(&start).unwrap() {
    return;
  }

  let mut is_null = true;
  for adj in graph.get(&start).unwrap() {
    if visited.contains(&adj.key) {
      continue;
    }

    if adj.is_term {
      is_null = false;
      break;
    }

    nullable_dfs(nullable_info, graph, adj.key.clone(), visited);

    if *nullable_info.get(&adj.key).unwrap() == false {
      is_null = false;
    }
  }

  if nullable_info.contains_key(&start) {
    *nullable_info.get_mut(&start).unwrap() = is_null;
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
    if !nt.is_start_term {
      continue;
    }

    for prod in &mut nt.productions {
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
  let mut graph: HashMap<String, HashSet<Node>> = HashMap::new();

  for nt in &mut *nts {
    if nt.is_nullable {
      continue;
    }

    if !graph.contains_key(&nt.name) {
      graph.insert(nt.name.clone(), HashSet::new());
    }

    for prod in &nt.productions {
      if prod.list.is_empty() {
        nt.is_nullable = true;
        break;
      }

      for token in &prod.list {
        if !graph.contains_key(&token.value) {
          graph.insert(token.value.clone(), HashSet::new());
        }

        let new_node = Node::new(token.value.clone(), !token.kind.eq("ID"));
        graph.get_mut(&nt.name).as_mut().unwrap().insert(new_node);
      }
    }
  }

  let mut nullable_info: HashMap<String, bool> = HashMap::new();
  for nt_inner in &mut *nts {
    nullable_info.insert(nt_inner.name.clone(), nt_inner.is_nullable);
  }

  for nt in &mut *nts {
    if nt.is_nullable {
      continue;
    }

    for prod in nt.productions.clone() {
      let mut prod_graph: HashMap<String, HashSet<Node>> = HashMap::new();

      for key in graph.keys() {
        if key.eq(&nt.name) {
          continue;
        }

        prod_graph.insert(key.clone(), HashSet::new());

        for node in graph.get(key).unwrap() {
          if node.key.eq(&nt.name) {
            continue;
          }

          prod_graph.get_mut(key).unwrap().insert(node.clone());
        }
      }

      prod_graph.insert(nt.name.clone(), HashSet::new());

      let adj_set = prod_graph.get_mut(&nt.name).unwrap();
      for token in &prod.list {
        let new_node = Node::new(token.value.clone(), !token.kind.eq("ID"));
        adj_set.insert(new_node);
      }

      let mut visited: Vec<String> = vec![];
      nullable_dfs(
        &mut nullable_info,
        &prod_graph,
        nt.name.clone(),
        &mut visited,
      );
      nt.is_nullable = *nullable_info.get(&nt.name).unwrap();

      if nt.is_nullable {
        break;
      }
    }
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
  for nt in nts {
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

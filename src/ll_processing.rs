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
use crate::error_handler::print_ambiguity;
use crate::productions::NonTerminal;

pub(crate) fn ll_process(non_terminals: &mut Vec<NonTerminal>) {
  predict_sets(non_terminals);
  find_ambiguities(non_terminals);
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
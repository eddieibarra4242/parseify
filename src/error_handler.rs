use std::collections::btree_set::Intersection;
use std::collections::HashSet;
use crate::lr_processing::{Action, State, StateTable};
use crate::parser::ParserError;
use crate::scanner::ScanError;

pub(crate) fn print_parse_err(file: String, error: ParserError) {
  let mut lines = file.split("\n");

  match error {
    ParserError::UnexpectedToken(token, expected) => {
      let line_num = token.span.start.line_num;
      let line = lines.nth(line_num - 1).unwrap();
      println!("{}", line);

      for i in 0..(token.span.end.col - 1) {
        if i > (token.span.start.col - 1) {
          print!("~");
        } else if i == (token.span.start.col - 1) {
          print!("^");
        } else {
          print!(" ");
        }
      }

      print!("\nUnexpected Token \"{}\" at line {}, expected ", token.value, line_num);
      for exp in &expected {
        if *exp == *expected.last().unwrap() {
          print!("{}", exp);
        } else {
          print!("{}, ", exp);
        }
      }

      println!();
    }
  }
}

pub(crate) fn print_scan_error(file: String, error: ScanError) {
  let mut lines = file.split("\n");

  match error {
    ScanError::UnexpectedChar(expected, seen, at) => {
      let line_num = at.line_num;
      let line = lines.nth(line_num - 1).unwrap();
      println!("{}", line);

      for i in 0..at.col {
        if i == (at.col - 1) {
          print!("^");
        } else {
          print!(" ");
        }
      }

      print!("\nUnexpected character \"{}\" at line {}", if seen == '\n' { "\\n".to_string() } else { seen.to_string() }, line_num);
      if expected != '_' {
        print!(", expected {}", expected);
      }

      println!();
    }
    ScanError::NoMoreChars(at) => {
      let line_num = at.line_num;
      println!("Line {} ended unexpectedly!", line_num);
      let line = lines.nth(line_num - 1).unwrap_or("").to_string();

      if line.is_empty() {
        println!("(empty)");
      } else {
        println!("{}", line);

        for i in 0..at.col {
          if i == (at.col - 1) {
            print!("^");
          } else {
            print!(" ");
          }
        }
      }

      println!();
    }
  }
}

pub(crate) fn print_ambiguity(nt_name: &String, intersection: Intersection<String>) {
  println!("Found ambiguities in {}:", nt_name);
  for amb in intersection {
    println!("  {}", amb);
  }

  println!();
}


pub(crate) fn terminal_box_string(state: &State, terminal: &String, common_actions: &Vec<Action>) -> String {
  let empty_vec = vec![];
  resolve_actions_to_string(state.actions.get(terminal).unwrap_or(&empty_vec), common_actions)
}

pub(crate) fn resolve_actions_to_string(actions: &Vec<Action>, common_actions: &Vec<Action>) -> String {
  let mut result = String::new();

  for action in common_actions {
    let action_string = resolve_action_to_string(action);
    result.push_str(action_string.as_str());
    result.push_str(", ");
  }

  for action in actions {
    let action_string = resolve_action_to_string(action);
    result.push_str(action_string.as_str());
    result.push_str(", ");
  }

  result.pop();
  result.pop();
  result
}

fn resolve_action_to_string(action: &Action) -> String {
  match action {
    Action::Accept => "accept".to_string(),
    Action::Shift(index) => format!("shift({})", index),
    Action::Reduce(term_list, nt) => {
      let mut res = format!("reduce({} ::= ", nt);

      for term in term_list {
        res.push_str(term.value.as_str());
        res.push(' ');
      }

      res.pop();
      res.push(')');
      res
    },
  }
}

fn non_terminal_box_string(state: &State, nt: &String) -> String {
  if !state.nt_state_transitions.contains_key(nt) {
    String::new()
  } else {
    format!("{}", state.nt_state_transitions.get(nt).unwrap())
  }
}

fn max_column_widths(rows: &Vec<Vec<String>>) -> Vec<usize> {
  let mut max_widths = vec![0usize; rows.first().unwrap().len()];

  for row in rows {
    for i in 0..row.len() {
      let entry_length = row[i].len();
      if entry_length > max_widths[i] {
        max_widths[i] = entry_length;
      }
    }
  }

  max_widths
}

pub(crate) fn print_state_table(state_table: &StateTable) {
  let middle_columns_header = Vec::from_iter(state_table.seen_terms.iter());
  let right_columns_header = Vec::from_iter(state_table.seen_non_terms.iter());
  let mut rows: Vec<Vec<String>> = vec![];

  for i in 0..state_table.states.len() {
    let state = &state_table.states[i];
    let mut entries = vec![];

    entries.push(format!("{}", i));

    for term in &middle_columns_header {
      entries.push(terminal_box_string(state, term, &state.common_actions));
    }

    for nt in &right_columns_header {
      entries.push(non_terminal_box_string(state, nt));
    }

    rows.push(entries);
  }

  let mut first_row = vec!["State".to_string()];
  for term in &middle_columns_header {
    if "EOF".eq(term.as_str()) {
      first_row.push("$".to_string());
      continue;
    }

    first_row.push(term.to_string());
  }

  for nt in &right_columns_header {
    first_row.push(nt.to_string());
  }

  rows.insert(0, first_row);

  print_box_table(rows, HashSet::from_iter([0, middle_columns_header.len()]));
}

fn print_box_separator(widths: &Vec<usize>, thick_line_indices: &HashSet<usize>, leftmost: &str, line: &str, thick_separator: &str, thin_separator: &str, rightmost: &str) {
  print!("{}", leftmost);
  for i in 0..widths.len() {
    let width = widths[i] + 2;

    for _ in 0..width {
      print!("{}", line);
    }

    if i == widths.len() - 1 {
      print!("{}", rightmost);
    } else if thick_line_indices.contains(&i) {
      print!("{}", thick_separator);
    } else {
      print!("{}", thin_separator);
    }
  }

  println!();
}

fn print_box_row(row: &Vec<String>, widths: &Vec<usize>, thick_line_indices: &HashSet<usize>, thick_separator: &str, thin_separator: &str) {
  print!("{}", thick_separator);
  for i in 0..widths.len() {
    let entry = &row[i];
    let width = widths[i] + 1 - entry.len();

    print!(" {}", entry);

    for _ in 0..width {
      print!(" ");
    }

    if i == widths.len() - 1 {
      print!("{}", thick_separator);
    } else if thick_line_indices.contains(&i) {
      print!("{}", thick_separator);
    } else {
      print!("{}", thin_separator);
    }
  }

  println!();
}

fn print_box_table(rows: Vec<Vec<String>>, thick_line_indices: HashSet<usize>) {
  let max_widths = max_column_widths(&rows);

  print_box_separator(&max_widths, &thick_line_indices, "╔", "═", "╦", "╤", "╗");

  for i in 0..rows.len() {
    print_box_row(&rows[i], &max_widths, &thick_line_indices, "║", "│");

    if i == 0 {
      print_box_separator(&max_widths, &thick_line_indices, "╠", "═", "╬", "╪", "╣");
    } else if i < rows.len() - 1 {
      print_box_separator(&max_widths, &thick_line_indices, "╟", "─", "╫", "┼", "╢");
    }
  }

  print_box_separator(&max_widths, &thick_line_indices, "╚", "═", "╩", "╧", "╝");
}
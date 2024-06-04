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

use std::collections::btree_set::Intersection;
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
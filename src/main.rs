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

use std::fs;
use clap::Parser;
use crate::error_handler::{print_parse_err, print_scan_error};
use crate::language::Language;
use crate::ll_processing::ll_process;
use crate::scanner::Scanner;

mod scanner;
mod productions;
mod language;
mod generator;
mod parser;
mod error_handler;
mod ll_processing;

/// Simple parser generator.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// Output file path
  #[arg(short, long)]
  output: Option<String>,

  /// input file path
  #[arg()]
  input: String,

  /// output file's language
  #[arg(short, long)]
  lang: Option<String>,

  /// Produce an LL(1) recursive descent parser
  #[arg(long)]
  ll: bool,

  /// Produce an LR(1) stack based parser
  #[arg(long)]
  lr: bool,
}

fn main() {
  let cli_args = Args::parse();

  if !(cli_args.ll ^ cli_args.lr) {
    println!("Please select only one type of parser to generate! Selected: LL(1) and LR(1).\n");
    return;
  }

  // fixme: maybe make an install location for language specifications.

  // default to rust language output.
  let lang_json = fs::read_to_string(cli_args.lang.unwrap_or("./langs/rust.json".to_string())).unwrap();
  let lang: Language = serde_json::from_str(lang_json.as_str()).unwrap();
  let file = fs::read_to_string(cli_args.input.clone()).expect(format!("Failed to open file: {}", cli_args.input).as_str());

  let mut scanner = Scanner::new(file.clone());
  let scanned_result = scanner.scan();

  if scanned_result.is_err() {
    print_scan_error(file, scanned_result.err().unwrap());
    return;
  }

  let tokens = scanned_result.unwrap();

  let mut parser = parser::Parser::new(tokens);
  let non_terminals_wrapped = parser.parse();

  if non_terminals_wrapped.is_err() {
    print_parse_err(file.clone(), non_terminals_wrapped.err().unwrap());
    return;
  }

  let mut non_terminals = non_terminals_wrapped.unwrap();
  productions::process(&mut non_terminals);

  if cli_args.lr {
    todo!("Implement LR processing...");
  } else {
    // Produce LL(1) parsers by default.
    ll_process(&mut non_terminals);
  }

  let output: String = generator::generate_parser(&non_terminals, &lang);
  let result = fs::write(cli_args.output.unwrap_or("./output.txt".to_string()), output);

  if result.is_err() {
    println!("Failed to write to file!");
  }
}

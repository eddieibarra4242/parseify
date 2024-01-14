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
use crate::language::Language;
use crate::scanner::Scanner;

mod scanner;
mod productions;
mod language;
mod generator;
mod parser;

/// Simple recursive descent parser generator.
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
}

fn main() {
  let cli_args = Args::parse();

  // fixme: maybe make an install location for language specifications.

  // default to rust language output.
  let lang_json = fs::read_to_string(cli_args.lang.unwrap_or("./langs/rust.json".to_string())).unwrap();
  let lang: Language = serde_json::from_str(lang_json.as_str()).unwrap();

  let mut scanner = Scanner::new(cli_args.input);
  let scanned_result = scanner.scan();

  if scanned_result.is_err() {
    println!("Scan error: {:?}", scanned_result.as_ref().err().unwrap());
    return;
  }

  let tokens = scanned_result.unwrap();

  let mut parser = parser::Parser::new(tokens);
  let mut non_terminals = parser.parse();

  productions::process(&mut non_terminals);

  let output: String = generator::generate_parser(&non_terminals, &lang);

  let result = fs::write(cli_args.output.unwrap_or("./output.txt".to_string()), output);

  if result.is_err() {
    println!("Failed to write to file!");
  }
}

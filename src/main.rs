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

use clap::Parser;
use crate::scanner::Scanner;

mod scanner;

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

    let mut scanner = Scanner::new(cli_args.input);
    let scanned_result = scanner.scan();

    if scanned_result.is_err() {
        println!("Scan error: {:?}", scanned_result.as_ref().err().unwrap());
        return;
    }

    let tokens = scanned_result.unwrap();
    println!("{:?}", tokens);
}

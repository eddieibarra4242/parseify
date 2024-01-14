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
use crate::scanner::ScanError::{NoMoreChars, UnexpectedChar};

#[derive(Debug, Copy, Clone)]
pub struct Coord {
  line_num: usize,
  col: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct Span {
  start: Coord,
  end: Coord,
}

#[derive(Debug, Clone)]
pub struct Token {
  pub(crate) kind: String,
  pub(crate) value: String,
  span: Span,
}

#[derive(Debug)]
pub enum ScanError {
  // expected, saw
  UnexpectedChar(char, char),
  NoMoreChars,
}

pub(crate) struct Scanner {
  file: String,
  next_char: usize,
  tokens: Vec<Token>,
  seen_newlines: usize,
  last_seen_newline_ndx: i64,
}

impl Scanner {
  pub(crate) fn new(file_path: String) -> Self {
    let file = fs::read_to_string(file_path.clone()).expect(format!("Failed to open file: {}", file_path).as_str());

    Scanner {
      file,
      next_char: 0,
      tokens: vec![],
      seen_newlines: 0,
      last_seen_newline_ndx: -1,
    }
  }

  pub(crate) fn scan(&mut self) -> Result<Vec<Token>, ScanError> {
    while self.has_next() {
      let start_of_token = self.next_char;
      let mut kind = String::new();

      let current = self.current()?;
      if current.is_whitespace() {
        self.whitespace()?;
        continue; // do not make whitespace tokens.
      } else if current == '/' {
        self.comment()?;
        continue; // do not make comment tokens.
      } else if current == '_' || current.is_alphabetic() {
        self.identifier()?;
        kind = "ID".to_string();
      } else if current == '"' || current == '\'' {
        self.literal()?;
        kind = "TERM".to_string();
      } else if current == ':' {
        // either ':' or '::=' is an equals token.
        self.match_char(':')?;

        if self.current()? == ':' {
          self.match_char(':')?;
          self.match_char('=')?;
        }

        kind = "EQUALS".to_string();
      } else if current == ';' {
        self.match_char(';')?;
        kind = "END".to_string();
      } else if current == '.' {
        self.match_char('.')?;
        kind = "END".to_string();
        // todo: maybe make newlines stop tokens as well...
      } else {
        return Err(UnexpectedChar('_', current));
      }

      let value = self.file[start_of_token..self.next_char].to_string();

      self.tokens.push(Token {
        kind,
        value,
        span: Span { start: self.index_to_coord(start_of_token), end: self.index_to_coord(self.next_char) },
      });
    }

    self.tokens.push(Token {
      kind: "EOF".to_string(),
      value: "".to_string(),
      span: Span { start: self.index_to_coord(self.next_char), end: self.index_to_coord(self.next_char) },
    });

    Ok(self.tokens.clone())
  }

  fn has_next(&self) -> bool {
    self.next_char < self.file.len()
  }

  fn current(&self) -> Result<char, ScanError> {
    if !self.has_next() {
      return Err(NoMoreChars);
    }

    match self.file.chars().nth(self.next_char) {
      None => Err(NoMoreChars),
      Some(character) => Ok(character)
    }
  }

  fn match_char(&mut self, expected: char) -> Result<(), ScanError> {
    if !self.has_next() {
      return Err(NoMoreChars);
    }

    if self.current()? != expected {
      return Err(UnexpectedChar(expected, self.current()?));
    }

    self.next_char += 1;

    Ok(())
  }

  fn index_to_coord(&self, index: usize) -> Coord {
    Coord {
      line_num: self.seen_newlines + 1,
      col: (index as i64 - self.last_seen_newline_ndx) as usize,
    }
  }

  fn comment(&mut self) -> Result<(), ScanError> {
    self.match_char('/')?;
    self.match_char('/')?;

    while self.has_next() && self.current()? != '\n' {
      self.match_char(self.current()?)?;
    }

    if self.has_next() && self.current()? == '\n' {
      self.seen_newlines += 1;
      self.last_seen_newline_ndx = self.next_char as i64;
      self.match_char('\n')?;
    } else {
      return Err(UnexpectedChar('\n', self.current()?)); // unicorn character
    }

    Ok(())
  }

  fn whitespace(&mut self) -> Result<(), ScanError> {
    while self.current()?.is_whitespace() {
      let current = self.current()?;

      if current == '\n' {
        self.seen_newlines += 1;
        self.last_seen_newline_ndx = self.next_char as i64;
      }

      self.match_char(current)?;
    }

    Ok(())
  }

  fn identifier(&mut self) -> Result<(), ScanError> {
    let mut current = self.current()?;

    if current == '_' || current.is_alphabetic() {
      self.match_char(current)?;
    }

    while self.current()? == '_' || self.current()?.is_alphabetic() || self.current()?.is_digit(10) {
      self.match_char(self.current()?)?;
    }

    Ok(())
  }

  fn literal(&mut self) -> Result<(), ScanError> {
    let mut current = self.current()?;

    if current == '"' {
      self.match_char('"')?;
    } else if current == '\'' {
      self.match_char('\'')?;
    } else {
      return Err(UnexpectedChar('"', current));
    }

    while self.current()? != '"' && self.current()? != '\'' {
      if self.current()? == '\n' {
        return Err(UnexpectedChar('"', '\n'));
      }

      self.match_char(self.current()?)?;
    }

    current = self.current()?;

    if current == '"' {
      self.match_char('"')?;
    } else if current == '\'' {
      self.match_char('\'')?;
    } else {
      return Err(UnexpectedChar('"', current));
    }

    Ok(())
  }
}

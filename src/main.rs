use std::io::Write;
use std::{env, error::Error, fs::File, str::Chars};
use walkdir::WalkDir;

// TODO :
//  - walk dir
//  - open c++ file

#[derive(Debug, Clone, Copy)]
enum TokenType {
    Space,
    Other,
    Comment,
    String,
    RawString,
    Identifier,
    And,
    Or,
    Eq,
    Neq,
    Xor,
    Null,
}

#[derive(Debug, Clone, Copy)]
struct Token<'a> {
    lexeme: &'a str,
    token_type: TokenType,
}

struct Scanner<'a> {
    src: &'a str,
    chars: Chars<'a>,
    prev_prev_char: Option<char>,
    prev_char: Option<char>,
    curr_char: Option<char>,
    next_char: Option<char>,
    start: usize,
    current: usize,
}

impl<'a> Scanner<'a> {
    fn new(src: &'a str) -> Self {
        let mut chars = src.chars();
        let next_char = chars.next();
        Self {
            src,
            chars,
            prev_prev_char: None,
            prev_char: None,
            curr_char: None,
            next_char,
            start: 0,
            current: 0,
        }
    }

    fn next(&mut self) -> Option<Token<'a>> {
        self.start = self.current;

        self.advance();
        if let Some(c) = self.curr_char {
            match c {
                ' ' | '\t' | '\r' | '\n' => Some(self.make_token(TokenType::Space)),
                '/' => Some(self.handle_comment()),
                '"' | '\'' => Some(self.handle_string(c)),
                _ => {
                    if c.is_alphabetic() {
                        let next = self.next_char.unwrap_or('\0');
                        if c == 'R' && next == '"' {
                            Some(self.handle_raw_string())
                        } else {
                            Some(self.handle_identifier())
                        }
                    } else {
                        Some(self.make_token(TokenType::Other))
                    }
                }
            }
        } else {
            None
        }
    }

    fn handle_identifier(&mut self) -> Token<'a> {
        while let Some(c) = self.next_char {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let str = &self.src[self.start..self.current];
        if str == "AND" {
            self.make_token(TokenType::And)
        } else if str == "OR" {
            self.make_token(TokenType::Or)
        } else if str == "EQ" {
            self.make_token(TokenType::Eq)
        } else if str == "NEQ" {
            self.make_token(TokenType::Neq)
        } else if str == "XOR" {
            self.make_token(TokenType::Xor)
        } else if str == "NULL" {
            self.make_token(TokenType::Null)
        } else {
            self.make_token(TokenType::Identifier)
        }
    }

    fn handle_comment(&mut self) -> Token<'a> {
        if let Some(next) = self.next_char {
            if next == '*' {
                loop {
                    self.advance();
                    if let Some(c) = self.curr_char {
                        if let Some(n) = self.next_char {
                            if c == '*' && n == '/' {
                                self.advance();
                                return self.make_token(TokenType::Comment);
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                // Multiline comment
            } else if next == '/' {
                // One line comment
                loop {
                    match self.next_char {
                        Some(c) => {
                            if c == '\n' {
                                self.advance();
                                return self.make_token(TokenType::Comment);
                            }
                            self.advance();
                        }
                        None => break,
                    }
                }
            }
        }

        self.make_token(TokenType::Other)
    }

    fn handle_string(&mut self, start_char: char) -> Token<'a> {
        loop {
            self.advance();
            match self.curr_char {
                Some(c) => {
                    if c == start_char {
                        // Hmm, `"\\"` was causing some problems :D
                        if self.prev_char.unwrap_or_default() == '\\'
                            && self.prev_prev_char.unwrap_or_default() == '\\'
                        {
                            break;
                        } else if self.prev_char.unwrap_or_default() != '\\' {
                            break;
                        }
                    }
                }
                None => break,
            }
        }
        self.make_token(TokenType::String)
    }

    fn handle_raw_string(&mut self) -> Token<'a> {
        // Consume the `"`
        self.advance();
        let start = self.current;

        // Look for the raw string name
        loop {
            self.advance();
            match self.curr_char {
                Some(c) => {
                    if c == '(' {
                        break;
                    }
                }
                None => return self.make_token(TokenType::Other), // Ill formed
            }
        }

        let raw_name = &self.src[start..self.current - 1];

        // Now, we look for `)raw_name"` ...
        let mut start = 0;
        loop {
            self.advance();

            match self.curr_char {
                Some(c) => {
                    if c == ')' {
                        start = self.current;
                    } else if c == '"' {
                        if raw_name == &self.src[start..self.current - 1] {
                            break;
                        }
                    }
                }
                None => return self.make_token(TokenType::Other), // Ill formed
            }
        }

        self.make_token(TokenType::RawString)
    }

    fn advance(&mut self) {
        // Advancing from one character is an error in the context of utf8
        self.current += self.curr_char.unwrap_or_default().len_utf8();

        self.prev_prev_char = self.prev_char;
        self.prev_char = self.curr_char;
        self.curr_char = self.next_char;
        self.next_char = self.chars.next();
    }

    fn make_token(&self, token_type: TokenType) -> Token<'a> {
        Token {
            lexeme: &self.src[self.start..self.current],
            token_type,
        }
    }
}

fn handle_file(str: &str) -> String {
    let mut scanner = Scanner::new(str);
    let mut out = String::with_capacity(str.len());
    while let Some(token) = scanner.next() {
        match token.token_type {
            TokenType::And => out.push_str("&&"),
            TokenType::Or => out.push_str("||"),
            TokenType::Eq => out.push_str("=="),
            TokenType::Neq => out.push_str("!="),
            TokenType::Xor => out.push_str("^"),
            TokenType::Null => out.push_str("nullptr"),
            _ => out.push_str(token.lexeme),
        }
    }

    out
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    if args.len() != 2 {
        eprintln!(
            "Please provide the root directory where the c++ files live (e.g. Vertigo/Development)"
        );
        return Ok(());
    }

    let root = args.nth(1).unwrap();

    for entry in WalkDir::new(root) {
        let path = entry?.into_path();
        // We ignore vrml as it contains weird stuff
        if path.to_str().unwrap().contains(&"Yacc") || path.to_str().unwrap().contains(&"Flex") {
            continue;
        }

        match path.extension() {
            Some(ext) => {
                if ext == "cpp" || ext == "h" || ext == "inl" {
                    println!("Converting {:?}...", path.as_os_str());
                    let content = std::fs::read_to_string(&path)?;
                    let converted = handle_file(content.as_str());

                    let mut file = File::create(&path)?;
                    file.write_all(converted.as_bytes())?;
                }
            }
            None => {}
        }
    }

    Ok(())
}

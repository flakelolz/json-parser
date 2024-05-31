use crate::reader::JsonReader;
use crate::value::Number;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
use std::iter::Peekable;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    CurlyOpen,
    CurlyClose,
    Quotes,
    Colon,
    String(String),
    Number(Number),
    ArrayOpen,
    ArrayClose,
    Comma,
    Boolean(bool),
    Null,
}

pub struct JsonTokenizer<T>
where
    T: Read + Seek,
{
    tokens: Vec<Token>,
    iterator: Peekable<JsonReader<T>>,
}

impl<T> JsonTokenizer<T>
where
    T: Read + Seek,
{
    pub fn new(reader: File) -> JsonTokenizer<File> {
        let json_reader = JsonReader::<File>::new(BufReader::new(reader));

        JsonTokenizer {
            tokens: vec![],
            iterator: json_reader.peekable(),
        }
    }

    pub fn from_bytes<'a>(input: &'a [u8]) -> JsonTokenizer<Cursor<&'a [u8]>> {
        let json_reader = JsonReader::<Cursor<&'a [u8]>>::from_bytes(input);

        JsonTokenizer {
            tokens: Vec::with_capacity(input.len()),
            iterator: json_reader.peekable(),
        }
    }

    pub fn tokenize_json(&mut self) -> Result<&[Token], ()> {
        while let Some(character) = self.iterator.peek() {
            match *character {
                '"' => {
                    // Pushed opening quote to output tokens list.
                    self.tokens.push(Token::Quotes);

                    // Skip quote token since we already added it to the tokens list.
                    let _ = self.iterator.next();

                    // Delegate parsing string value to a separate function.
                    // The function should also take care of advancing the iterator properly
                    let string = self.parse_string();

                    // Push parsed string to ouput tokens list.
                    self.tokens.push(Token::String(string));

                    // Pushed closing quote to output tokens list.
                    self.tokens.push(Token::Quotes);
                }
                '-' | '0'..='9' => {
                    let number = self.parse_number()?;
                    self.tokens.push(Token::Number(number));
                }
                // Match `t` character which indicates beginning of a boolean literal.
                't' => {
                    // Advance iterator by 1.
                    let _ = self.iterator.next();

                    // Assert next character is `r` while advancing the iterator by 1.
                    assert_eq!(Some('r'), self.iterator.next());
                    // Assert next character is `u` while advancing the iterator by 1.
                    assert_eq!(Some('u'), self.iterator.next());
                    // Assert next character is `e` while advancing the iterator by 1.
                    assert_eq!(Some('e'), self.iterator.next());

                    // Push the literal value to token list.
                    self.tokens.push(Token::Boolean(true))
                }
                // Match `f` character which indicates beginning of a boolean literal.
                'f' => {
                    // Advance iterator by 1.
                    let _ = self.iterator.next();

                    // Assert next character is `a` while advancing the iterator by 1.
                    assert_eq!(Some('a'), self.iterator.next());
                    // Assert next character is `l` while advancing the iterator by 1.
                    assert_eq!(Some('l'), self.iterator.next());
                    // Assert next character is `s` while advancing the iterator by 1.
                    assert_eq!(Some('s'), self.iterator.next());
                    // Assert next character is `e` while advancing the iterator by 1.
                    assert_eq!(Some('e'), self.iterator.next());

                    // Push the literal value to token list.
                    self.tokens.push(Token::Boolean(false));
                }
                // Match `n` character which indicates beginning of a null literal.
                'n' => {
                    // Advance iterator by 1.
                    let _ = self.iterator.next();

                    // Assert next character is `u` while advancing the iterator by 1.
                    assert_eq!(Some('u'), self.iterator.next());
                    // Assert next character is `l` while advancing the iterator by 1.
                    assert_eq!(Some('l'), self.iterator.next());
                    // Assert next character is `l` while advancing the iterator by 1.
                    assert_eq!(Some('l'), self.iterator.next());

                    // Push null literal value to output tokens list.
                    self.tokens.push(Token::Null);
                }
                // Delimeters
                '{' => {
                    self.tokens.push(Token::CurlyOpen);
                    let _ = self.iterator.next();
                }
                '}' => {
                    self.tokens.push(Token::CurlyClose);
                    let _ = self.iterator.next();
                }
                '[' => {
                    self.tokens.push(Token::ArrayOpen);
                    let _ = self.iterator.next();
                }
                ']' => {
                    self.tokens.push(Token::ArrayClose);
                    let _ = self.iterator.next();
                }
                ',' => {
                    self.tokens.push(Token::Comma);
                    let _ = self.iterator.next();
                }
                ':' => {
                    self.tokens.push(Token::Colon);
                    let _ = self.iterator.next();
                }
                '\0' => break,
                other => {
                    if !other.is_ascii_whitespace() {
                        panic!("Unexpected token encountered: {other}")
                    } else {
                        self.iterator.next();
                    }
                }
            }
        }
        Ok(&self.tokens)
    }

    fn parse_string(&mut self) -> String {
        // Create new vector to hold parsed characters.
        let mut string_characters = Vec::new();

        // Take each character by reference so that they aren't moved out of the iterator, which
        // will require you to move the iterator into this function.
        for character in self.iterator.by_ref() {
            // If it encounters a closing `"`, break out of the loop as the string has ended.
            if character == '"' {
                break;
            }

            // Continue pushing to the vector to build the string.
            string_characters.push(character);
        }

        // Create a string out of the character iterator and return it.
        String::from_iter(string_characters)
    }

    fn parse_number(&mut self) -> Result<Number, ()> {
        // Store parsed number characters.
        let mut number_characters = Vec::new();

        // Stores wether the digit being parsed is a `.` character making it a decimal.
        let mut is_decimal = false;

        // Stores the characters after an apsilon character `e` or `E` to indicate the exponential
        // value.
        let mut epsilon_characters = Vec::new();

        // Stores wether the digit being parsed is part of the epsilon characters.
        let mut is_epsilon_characters = false;

        while let Some(character) = self.iterator.peek() {
            match character {
                '-' => {
                    if is_epsilon_characters {
                        // If it's parsing epsilon characters, push it to the epsilon character
                        // set.
                        epsilon_characters.push('-');
                    } else {
                        // Otherwise, push it to the normal character set.
                        number_characters.push('-');
                    }

                    // Advance the iterator by 1.
                    let _ = self.iterator.next();
                }
                // Match a positive sign, which can be trated as  redundant and ignored since
                // positive is the default.
                '+' => {
                    // Advance the iterator by 1.
                    let _ = self.iterator.next();
                }
                // Match any digit between 0 and 9, and store it into the `digit` variable.
                digit @ '0'..='9' => {
                    if is_epsilon_characters {
                        // If it's parsing epsilon characters, push it to the epsilon character
                        // set.
                        epsilon_characters.push(*digit);
                    } else {
                        // Otherwise, push it to the normal character set.
                        number_characters.push(*digit);
                    }

                    // Advance the iterator by 1.
                    let _ = self.iterator.next();
                }
                '.' => {
                    // Push the decimal character to numbers character set.
                    number_characters.push('.');

                    // Set the current state of number being decimal to true.
                    is_decimal = true;

                    // Advance the iterator by 1.
                    let _ = self.iterator.next();
                }
                // Match any of the characters that can signify end of the number literal value.
                // This can be a comma which separated key-value pair, closing object character,
                // closing array character, or a `:` which separates a key from its value.
                '}' | ',' | ']' | ':' => {
                    break;
                }
                // Match the epsilon character which indicates that the number is in scrientific
                // notation.
                'e' | 'E' => {
                    // Panic if it's already parsing an exponential number since this would mean
                    // there are 2 epsilon characters which is invalid.
                    if is_epsilon_characters {
                        panic!("Unexpected character while parsing number: {character}. Double epsilon characters encountered");
                    }

                    // Set the current state of number being in scientific notation to true.
                    is_epsilon_characters = true;

                    // Advance the iterator by 1.
                    let _ = self.iterator.next();
                }
                // Panic if any other character is encountered.
                other => {
                    if !other.is_ascii_whitespace() {
                        panic!("Unexpected character while parsing number: {character}")
                    } else {
                        self.iterator.next();
                    }
                }
            }
        }
        if is_epsilon_characters {
            // if the number is an exponential, perform the calculations to convert it to a
            // floating point number in Rust.

            // Parse base as floating point number.
            let base: f64 = String::from_iter(number_characters).parse().unwrap();

            // Parse exponential as floating point number;
            let exponential: f64 = String::from_iter(epsilon_characters).parse().unwrap();

            // Return the final computed decial number.
            Ok(Number::F64(base * 10_f64.powf(exponential)))
        } else if is_decimal {
            // if the number is a decimal, parse it as a floating point number in rust.
            Ok(Number::F64(
                String::from_iter(number_characters).parse::<f64>().unwrap(),
            ))
        } else {
            // Parse the number as an integer in Rust.
            Ok(Number::I64(
                String::from_iter(number_characters).parse::<i64>().unwrap(),
            ))
        }
    }
}

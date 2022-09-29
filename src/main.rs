use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use serde_json::Map;
use thiserror::Error;

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();

    assert_eq!(args.len(), 2, "Expected 1 argument: <path to event data>");

    let path = &args[1];
    let json = parse_event_data(path)?;

    println!("{}", serde_json::to_string_pretty(&json)?);

    Ok(())
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("invalid key")]
    InvalidKey,
    #[error("invalid value")]
    InvalidValue,
}

/// Parses event data from a file
fn parse_event_data(file_path: &str) -> Result<serde_json::Value, anyhow::Error> {
    let mut file = BufReader::new(File::open(file_path).expect("Failed to open file"));

    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let mut it = data.chars().peekable();
    let mut pairs = Map::new();
    while let Some(&c) = it.peek() {
        // Skip whitespace before a possible key.
        if c.is_whitespace() {
            it.next();
            continue;
        }

        let key = parse_key(&mut it)?;
        let value = parse_value(&mut it)?;

        pairs.insert(key, value.into());
    }

    Ok(serde_json::Value::Object(pairs))
}

/// Skips all leading white spaces of the given iterator
fn skip_leading_whitespace(it: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = it.peek() {
        if !c.is_whitespace() {
            break;
        }
        it.next();
    }
}

/// Parses a value
///
/// Leading whitespace are ignored.
fn parse_value(it: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, ParseError> {
    skip_leading_whitespace(it);

    // Check for opening double quotes.
    if let Some(&c) = it.peek() {
        if c == '"' {
            it.next();
        } else {
            return Err(ParseError::InvalidValue);
        }
    } else {
        return Err(ParseError::InvalidValue);
    }

    let mut value = String::new();

    // Parse until we encounter closing double quotes.
    while let Some(&c) = it.peek() {
        if c == '"' {
            it.next();
            break;
        }

        // Handle escaped double quote.
        if c == '\\' {
            // Encountered an escaped character.
            // We assume the character to be double quotes.
            it.next();
            if let Some(&c) = it.peek() {
                if c == '"' {
                    value.push(c);
                    it.next();
                    continue;
                }
                return Err(ParseError::InvalidValue);
            } else {
                return Err(ParseError::InvalidValue);
            }
        }

        value.push(c);
        it.next();
    }

    Ok(value)
}

/// Parses a key
///
/// Leading whitespace are ignored.
fn parse_key(it: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, ParseError> {
    skip_leading_whitespace(it);

    let mut key = String::new();

    while let Some(&c) = it.peek() {
        if c == ':' {
            it.next();
            break;
        }

        key.push(c);
        it.next();
    }

    if it.peek().is_none() {
        return Err(ParseError::InvalidKey);
    }

    Ok(key)
}

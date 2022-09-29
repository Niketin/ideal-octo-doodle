use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use serde_json::Map;
use thiserror::Error;

// Part B hint is "Hello, try XOR with 0x17F".
const XOR_KEY: u16 = 0x17F;

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();

    assert_eq!(args.len(), 2, "Expected 1 argument: <path to event data>");

    let path = &args[1];
    let mut object = parse_event_data(path)?;

    let fifth_value = figure_fifth_value(&object);
    let map = object.as_object_mut().unwrap();
    let value_str = format!("0x{:x}", fifth_value);
    map.insert("five".to_string(), value_str.into());
    println!("{}", serde_json::to_string_pretty(&object)?);

    Ok(())
}

/// Computes the fifth value and returns it
///
///
///
/// Prints a debug print to stderr, which helped me to figure out what the fifth value could be.
/// Example output:
/// ```
/// one   0x154 0b101010100 43 +
/// two   0x150 0b101010000 47 /
/// three 0x14A 0b101001010 53 5
/// four  0x144 0b101000100 59 ;
/// ```
///
/// It seems to be an increasing sequence of integers (43, 47, 53, 59, ...)
/// Increments are 4, 6, 6, ...
///
/// Simplest rule I could figure out is the following is as follows.
///     x_{i+2} = x_{i+1} + index_of_first_mismatching_bit(x_i, x_{i+1}) * (i - 1)
/// Here the function index_of_first_mismatching_bit returns an index starting from 1.
/// Also i starts from 1.
/// Example:
/// x_3 = x_3 + index_of_first_mismatching_bit(x_1, x_2) * 2
///     = 47 + index_of_first_mismatching_bit(43, 47) * 2
///     = 47 + 3 * 2
///     = 53
/// x_4 = x_3 + index_of_first_mismatching_bit(x_2, x_3) * 3
///     = 53 + index_of_first_mismatching_bit(47, 53) * 3
///     = 53 + 2 * 3
///     = 59
/// x_5 = x_4 + index_of_first_mismatching_bit(x_3, x_4) * 4
///     = 59 + index_of_first_mismatching_bit(53, 59) * 4
///     = 59 + 2 * 4
///     = 67
fn figure_fifth_value(object: &serde_json::Value) -> u16 {
    let object_members = object.as_object().expect("Given value was not an object.");

    let keys = vec!["one", "two", "three", "four"];
    let mut values = vec![];
    for key in keys {
        let value_str = object_members[key]
            .as_str()
            .unwrap_or_else(|| panic!("Unexpected value for key \"{}\"", key));
        let value_str_trimmed = value_str.trim_start_matches("0x");
        let value = u16::from_str_radix(value_str_trimmed, 16)
            .unwrap_or_else(|_| panic!("Unexpected value for key \"{}\"", key));
        let value_xor = value ^ XOR_KEY;
        eprintln!(
            "{:5} {} {:#b} xorred:{:#b} {} {}",
            key,
            value_str,
            value,
            value_xor,
            value_xor,
            char::from_u32(value_xor as u32).expect("TODO")
        );
        values.push(value_xor);
    }

    // Compute the fifth value.
    let three = values[2];
    let four = values[3];
    let index_of_first_mismatching_bit = |a: u16, b: u16| (a ^ b).trailing_zeros() as u16 + 1;
    (four + index_of_first_mismatching_bit(three, four) * 4) ^ XOR_KEY
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

extern crate num_complex;
extern crate scpl;
extern crate regex;

type TYPE = f64;

use num_complex::Complex;
use scpl::complex_func::ComplexNode;
use scpl::complex_plane::ComplexPlane;
use scpl::complex_func::complex_definition::ComplexDefinition;
use self::regex::Regex;
use std::thread;
use std::error::Error;
use std::fmt;

pub enum Command {
    DEFINE { left: String, right: String },
    EXPRESSION(ComplexNode<TYPE>),
    IF {
        expression: Box<Command>,
        inside: Box<Command>,
        or: Box<Command>,
    },
    ELSE { inside: Box<Command> },
    WHILE {
        expression: Option<Box<Command>>,
        inside: Vec<Command>,
    },
    BLOCK(Vec<Command>),
    DO_NOTHING,
}
#[derive(Debug)]
pub enum ScparseError {
    BracketNotClosed,
    UNKNOWN(String),
}
impl fmt::Display for ScparseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ScparseError::BracketNotClosed => write!(f, "bracket is not closed"),
            &ScparseError::UNKNOWN(ref s) => write!(f, "{}", s),
        }
    }
}
impl Error for ScparseError {
    fn description(&self) -> &str {
        "ScparseError"
    }
}
fn if_parse<'a>(script: &'a str, j: i32) -> Result<(Command, &'a str), ScparseError> {
    let regex_if = Regex::new("^if").unwrap();
    let regex_elif = Regex::new("^elif").unwrap();
    let regex_else = Regex::new("^else").unwrap();
    if regex_if.is_match(script) {
        let tmp: Vec<&str> = regex_if.splitn(script, 2).collect();
        let script = tmp[1];
        let tmp: Vec<&str> = script.splitn(2, "{").collect();
        let exp: &str = tmp[0];
        let mut counter = 1;
        let mut index = 0;
        let mut block = String::from("{");
        for c in tmp[1].chars() {
            index += 1;
            block.push(c);
            if c == '{' {
                counter += 1;
            } else if c == '}' {
                counter -= 1;
            }
            if counter <= 0 {
                let exp = parse(exp, j)?.into_iter().nth(0).unwrap();
                let block = parse(&block, j)?.into_iter().nth(0).unwrap();
                let (or, next) = if_parse(&script[index..], j)?;
                return Ok((
                    Command::IF {
                        expression: Box::new(exp),
                        inside: Box::new(block),
                        or: Box::new(or),
                    },
                    next,
                ));
            }
        }
        return Err(ScparseError::BracketNotClosed);
    } else if regex_elif.is_match(script) {
        let tmp: Vec<&str> = regex_elif.splitn(script, 2).collect();
        let script = tmp[1];
        let tmp: Vec<&str> = script.splitn(2, "{").collect();
        let exp: &str = tmp[0];
        let mut counter = 1;
        let mut index = 0;
        let mut block = String::from("{");
        for c in tmp[1].chars() {
            index += 1;
            block.push(c);
            if c == '{' {
                counter += 1;
            } else if c == '}' {
                counter -= 1;
            }
            if counter <= 0 {
                let exp = parse(exp, j)?.into_iter().nth(0).unwrap();
                let block = parse(&block, j)?.into_iter().nth(0).unwrap();
                let (or, next) = if_parse(&script[index..], j)?;
                return Ok((
                    Command::IF {
                        expression: Box::new(exp),
                        inside: Box::new(block),
                        or: Box::new(or),
                    },
                    next,
                ));
            }
        }
        return Err(ScparseError::BracketNotClosed);
    } else if regex_else.is_match(script) {
        let tmp: Vec<&str> = regex_elif.splitn(script, 2).collect();
        let script = tmp[1];
        let tmp: Vec<&str> = script.splitn(2, "{").collect();
        let mut counter = 1;
        let mut index = 0;
        let mut block = String::from("{");
        for c in tmp[1].chars() {
            index += 1;
            block.push(c);
            if c == '{' {
                counter += 1;
            } else if c == '}' {
                counter -= 1;
            }
            if counter <= 0 {
                let block = parse(&block, j)?.into_iter().nth(0).unwrap();
                return Ok((
                    Command::ELSE { inside: Box::new(block) },
                    &script[index..],
                ));
            }
        }
        return Err(ScparseError::BracketNotClosed);
    } else {
        return Ok((Command::DO_NOTHING, script));
    }

}
pub fn parse(script: &str, j: i32) -> Result<Vec<Command>, ScparseError> {
    let script = script.trim();
    if script.is_empty() {
        return Ok(Vec::new());
    }
    let mut vecter: Vec<Command> = Vec::new();
    let regex_if = Regex::new("^if").unwrap();
    let regex_while = Regex::new("^while").unwrap();
    let regex_output = Regex::new("^output").unwrap();
    let regex_return = Regex::new("^return").unwrap();
    if script.starts_with('{') {
        let left = 1;
        let mut counter = 1;
        let mut right = left;
        let char_vec = (&script[left..]).chars();
        for c in char_vec {
            if c == '{' {
                counter += 1;
            } else if c == '}' {
                counter -= 1;
            }
            if counter <= 0 {
                let block = &script[left..right];
                let next = &script[(right + 1)..];
                if j > 1 {
                    let owned_block = block.to_owned();
                    let handler = thread::spawn(move || parse(&owned_block, j - 1));
                    let mut next = parse(next, j - 1)?;
                    let block = handler.join().unwrap()?;
                    vecter.push(Command::BLOCK(block));
                    vecter.append(&mut next);
                    return Ok(vecter);
                } else {
                    let block = parse(block, 1)?;
                    let mut next = parse(next, 1)?;
                    vecter.push(Command::BLOCK(block));
                    vecter.append(&mut next);
                    return Ok(vecter);
                }
            }
            right += 1;
        }
        return Err(ScparseError::BracketNotClosed);
    } else if regex_if.is_match(script) {
        let (cmd, next) = if_parse(script, j)?;
        vecter.push(cmd);
        vecter.append(&mut parse(next, j)?);
    } else if regex_while.is_match(script) {
        let script = script.trim_left_matches("while");
    }
    Ok(vecter)
}








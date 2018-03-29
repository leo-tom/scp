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

#[derive(Debug)]
pub enum Command {
    DEFINE (String,Box<Command>),
    DEFINE_SIMPLE (String,ComplexNode<TYPE>),
    EXPRESSION(ComplexNode<TYPE>),
    IF {
        expression: Box<Command>,
        inside: Box<Command>,
        or: Box<Command>,
    },
    ELIF {
        expression: Box<Command>,
        inside: Box<Command>,
        or: Box<Command>,
    },
    ELSE (Box<Command>),
    WHILE {
        expression: Box<Command>,
        inside: Box<Command>,
    },
    BLOCK(Vec<Command>),
    DO_NOTHING,
}
#[derive(Debug)]
pub enum ScparseError {
    BracketNotClosed,
    NoRightHandSide(String),
    UNKNOWN(String),
}
impl fmt::Display for ScparseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ScparseError::BracketNotClosed => write!(f, "bracket is not closed"),
            &ScparseError::NoRightHandSide(ref right_hand_side) => write!(f,
                "There is no right hand side of {}",right_hand_side),
            &ScparseError::UNKNOWN(ref s) => write!(f, "{}", s),
        }
    }
}
impl Error for ScparseError {
    fn description(&self) -> &str {
        "ScparseError"
    }
}
impl Command {
    fn _to_string(&self,depth:u32) -> String {
        let mut buff = String::new();
        match self {
            &Command::BLOCK(ref vec) => {
                let depth = depth + 1;
                for _ in 0..depth {buff.push_str("    ");}
                buff.push('{');
                buff.push('\n');
                for cmd in vec {
                    buff.push_str(&cmd._to_string(depth+1));
                }
                for _ in 0..depth {buff.push_str("    ");}
                buff.push('}');
            }
            &Command::DEFINE(ref left,ref right) => {
                for _ in 0..depth {buff.push_str("    ");}
                buff.push_str(&format!("DEFINE {} as",left));
                buff.push('\n');
                buff.push_str(&right._to_string(depth+1));
            }
            &Command::DEFINE_SIMPLE(ref left,ref right) => {
                for _ in 0..depth {buff.push_str("    ");}
                buff.push_str(&format!("DEFINE {} as ",left));
                buff.push_str(&right.to_string());
            }
            &Command::IF{ref expression,ref inside,ref or} => {
                for _ in 0..depth {buff.push_str("    ");}
                buff.push_str(&format!("IF {} ",expression));
                buff.push('\n');
                buff.push_str(&inside._to_string(depth+1));
                buff.push_str(&or._to_string(depth));
            }
            &Command::ELIF{ref expression,ref inside,ref or} => {
                for _ in 0..depth {buff.push_str("    ");}
                buff.push_str(&format!("ELIF {} ",expression));
                buff.push('\n');
                buff.push_str(&inside._to_string(depth+1));
                buff.push_str(&or._to_string(depth));
            }
            &Command::ELSE(ref cmd) => {
                for _ in 0..depth {buff.push_str("    ");}
                buff.push_str("ELSE");
                buff.push('\n');
                buff.push_str(&cmd._to_string(depth+1));
            }
            &Command::EXPRESSION(ref exp)=> {
                for _ in 0..depth {buff.push_str("    ");}
                buff.push_str(&format!("EXP {}",exp));
            }
            &Command::WHILE{ref expression,ref inside} => {
                for _ in 0..depth {buff.push_str("    ");}
                buff.push_str(&format!("WHILE {} ",expression));
                buff.push('\n');
                buff.push_str(&inside._to_string(depth+1));
            }
            &Command::DO_NOTHING => {
                for _ in 0..depth {buff.push_str("    ");}
                buff.push_str("#DO_NOTHNG");
            }
        }
        buff.push('\n');
        buff
    }
    pub fn to_string(&self) -> String {
        self._to_string(0)
    }
}
impl fmt::Display for Command {

    fn  fmt(&self,f: &mut fmt::Formatter) -> fmt::Result{
        write!(f,"{}",self.to_string())
    }
}
fn get_block<'a>(code : &'a str) -> Result<(&'a str,&'a str),ScparseError> {
    /*Split code like (Block,Code following code)*/
    /*Assume the argument doesn't have first {*/
    let mut index = 0;
    let mut counter = 1;
    for c in code.chars() {
        if c == '{' {
            counter += 1;
        }else if c == '}' {
            counter -= 1;
        }
        index += 1;
        if counter <= 0 {
            return Ok( (&code[..index],&code[index..]) );
        }
    }
    return Err(ScparseError::BracketNotClosed);
}

pub fn parse(script: &str, j: u32) -> Result<Vec<Command>, ScparseError> {
    let script = script.trim();
    if script.is_empty() {
        return Ok(Vec::new());
    }
    let mut vecter: Vec<Command> = Vec::new();
    let regex_output = Regex::new("^output").unwrap();
    let regex_return = Regex::new("^return").unwrap();
    if script.starts_with('{') {
        let (block,following) = get_block(&script[1..])?;
        if j > 1 {
            let owned_block = block.to_owned();
            let handler = thread::spawn(move || parse(&owned_block, j - 1));
            let mut following = parse(following, j - 1)?;
            let block = handler.join().unwrap()?;
            vecter.push(Command::BLOCK(block));
            vecter.append(&mut following);
            return Ok(vecter);
        } else {
            let block = parse(block, 1)?;
            let mut following = parse(following, 1)?;
            vecter.push(Command::BLOCK(block));
            vecter.append(&mut following);
            return Ok(vecter);
        }
    } else if script.starts_with("if") {
        let script = script.trim_left_matches("if");
        let splitted:Vec<Command> = parse(script, j)?;
        let mut iter = splitted.into_iter();
        let exp = iter.next().unwrap();
        let inside = iter.next().unwrap();
        let next = iter.next().unwrap();
        match next {
            x @ Command::ELIF{expression:_,inside:_,or:_} => {
                let cmd = Command::IF{
                    expression:Box::new(exp),
                    inside : Box::new(inside),
                    or:Box::new(x),
                };
                vecter.push(cmd);
            },
            x @  Command::ELSE(_) => {
                let cmd = Command::IF{
                    expression:Box::new(exp),
                    inside : Box::new(inside),
                    or:Box::new(x),
                };
                vecter.push(cmd);
            },
            x @ _ => {
                let cmd = Command::IF{
                    expression:Box::new(exp),
                    inside : Box::new(inside),
                    or : Box::new(Command::DO_NOTHING),
                };
                vecter.push(cmd);
                vecter.push(x);
            }
        }
        for cmd in iter {
            vecter.push(cmd);
        }
    } else if script.starts_with("elif") {
        let script = script.trim_left_matches("elif");
        let splitted:Vec<Command> = parse(script, j)?;
        let mut iter = splitted.into_iter();
        let exp = iter.next().unwrap();
        let inside = iter.next().unwrap();
        let next = iter.next().unwrap();
        match next {
            x @ Command::ELIF{expression:_,inside:_,or:_} => {
                let cmd = Command::ELIF{
                    expression:Box::new(exp),
                    inside : Box::new(inside),
                    or:Box::new(x),
                };
                vecter.push(cmd);
            },
            x @  Command::ELSE(_) => {
                let cmd = Command::ELIF{
                    expression:Box::new(exp),
                    inside : Box::new(inside),
                    or:Box::new(x),
                };
                vecter.push(cmd);
            },
            x @ _ => {
                let cmd = Command::ELIF{
                    expression:Box::new(exp),
                    inside : Box::new(inside),
                    or : Box::new(Command::DO_NOTHING),
                };
                vecter.push(cmd);
                vecter.push(x);
            }
        }
        for cmd in iter {
            vecter.push(cmd);
        }
    }else if script.starts_with("else") {
        let script = script.trim_left_matches("else");
        let splitted:Vec<Command> = parse(script, j)?;
        let mut iter = splitted.into_iter();
        let inside = iter.next().unwrap();
        let cmd = Command::ELSE(Box::new(inside));
        vecter.push(cmd);
        for cmd in iter {
            vecter.push(cmd);
        }
    }else if script.starts_with("while") {
        let script = script.trim_left_matches("while");
        let splitted:Vec<Command> = parse(script, 1)?;
        let mut iter = splitted.into_iter();
        let exp = iter.next().unwrap();
        let inside = iter.next().unwrap();
        let cmd = Command::WHILE{
            expression:Box::new(exp),
            inside: Box::new(inside)
        };
        vecter.push(cmd);
        for cmd in iter {
            vecter.push(cmd);
        }
    } else if script.starts_with('#') {
        let splitted:Vec<&str> = script.splitn(2,'\n').collect();
        vecter.append(&mut parse(splitted[1], j)?);
    }
    else{
        /*This is expression*/
        let (exp,following) = {
            let mut index = 0;
            let mut retval = (script,"");
            for c in script.chars() {
                match c {
                    '\n' | ';' => {
                        retval = (&script[..index],&script[(index+1)..]);
                        break;
                    }
                    '{' | '#' => {
                        retval = (&script[..index],&script[index..]);
                        break;
                    }
                    _ => index += 1,
                }
            }
            retval
        };
        let handler =  if j >= 2 {
            let following = String::from(following);
            Some(thread::spawn(move || parse(&following, j-1)))
        }else{
            None
        };
        if exp.contains('=') {
            let splitted:Vec<&str> = exp.splitn(2,'=').collect();
            let left = splitted[0];
            let vec = parse(splitted[1], 1)?;
            if vec.len() < 1 {
                return Err(ScparseError::NoRightHandSide(left.to_owned()));
            }
            let mut iter = vec.into_iter();
            let right = iter.next().unwrap();
            let def = match right {
                Command::EXPRESSION(x) => {
                    Command::DEFINE_SIMPLE(left.to_owned(),x)
                }
                _ => Command::DEFINE(left.to_owned(),Box::new(right))
            };
            vecter.push(def);
        }else{
            match ComplexNode::parse(exp) {
                Some(x) => vecter.push(Command::EXPRESSION(*x)),
                None    => ()
            }
        }
        match handler {
            Some(x) => vecter.append(&mut x.join().unwrap()?),
            None => vecter.append(&mut parse(&following, 1)?)
        }
    }
    Ok(vecter)
}












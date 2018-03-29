extern crate num_complex;
extern crate scpl;
extern crate getopts;

mod parser;

use getopts::Options;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use num_complex::Complex;
use scpl::complex_func::ComplexNode;
use scpl::complex_plane::ComplexPlane;
use scpl::complex_func::complex_definition::ComplexDefinition;
use parser::Command;

type TYPE = f64;

const THREAD_COUNT:u32 = 2;




static PROGRAM_NAME: &str = "scp";
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("e", "expression", "execute script", "EXPRESSION");
    opts.optopt("i", "input", "specify input files", "FILE");
    opts.optopt("o", "output", "specify output file", "FILE");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        println!(
            "{}",
            opts.usage(&format!("Usage: {} FILE [OPTIONS]", PROGRAM_NAME))
        );
        return;
    }
    let mut commands:Vec<Command> = Vec::new();
    if !matches.opt_present("i") {
        let mut buff = String::new();
        std::io::stdin().read_to_string(&mut buff).unwrap();
        match parser::parse(&buff, THREAD_COUNT) {
            Ok(mut x) => commands.append(&mut x),
            Err(e) => panic!("{}",e),
        }
    }
    for file_name in matches.opt_strs("i") {
        let mut file = File::open(file_name).unwrap();
        let mut buff = String::new();
        file.read_to_string(&mut buff).unwrap();
        match parser::parse(&buff, THREAD_COUNT) {
            Ok(mut x) => commands.append(&mut x),
            Err(e) => panic!("{}",e),
        }
    }
    for expression in matches.opt_strs("e") {
        parser::parse(&expression, 1).unwrap();
    }

    for cmd in commands {
        println!("{}",cmd );
    }
}

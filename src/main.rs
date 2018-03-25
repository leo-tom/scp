extern crate num_complex;
extern crate scpl;
extern crate getopts;


use getopts::Options;
use std::env;
use num_complex::Complex;
use scpl::complex_func::ComplexNode;
use scpl::complex_plane::ComplexPlane;
use scpl::complex_func::complex_definition::ComplexDefinition;

type TYPE = f64;

mod parser;

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
    for expression in matches.opt_strs("e") {
        parser::parse(&expression, 1);
    }
}

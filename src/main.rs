extern crate combine;

mod compiler;
mod primitive;
mod reader;
mod value;
mod vm;

use std::env::args;
use std::fs::File;
use std::io::Read;
use primitive::define_primitives;
use reader::read;
use vm::Machine;

fn main() {
    let mut lib = File::open("./lib/base.scm").unwrap();
    let mut buf = String::new();
    lib.read_to_string(&mut buf).unwrap();
    let (ast, _) = read(&buf).unwrap();
    let mut global = define_primitives();
    for exp in ast {
        let code = exp.compile(&global).unwrap();
        Machine::run(Vec::new(), code, &mut global).unwrap();
    }

    let mut file = File::open(args().nth(1).unwrap()).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let (ast, _) = read(&buf).unwrap();
    for exp in ast {
        let code = exp.compile(&global).unwrap();
        Machine::run(Vec::new(), code, &mut global).unwrap();
    }
}

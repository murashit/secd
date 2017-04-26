use std::fmt;
use std::rc::Rc;
use vm::{Code, Env};
use compiler::Ast;
use reader::read;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Integer(i32),
    Symbol(String),
    Cell(Rc<(Value, Value)>),
    Primitive(fn(Vec<Value>) -> Result<Value, String>),
    Closure(Code, Env),
    Macro(Code, Env),
    Undefined,
}

impl Value {
    pub fn to_ast(&self) -> Ast {
        let string = format!("{}", self);
        let (ast, _) = read(&string).unwrap();
        ast[0].to_owned()
    }

    pub fn cons(car: Value, cdr: Value) -> Value {
        Value::Cell(Rc::new((car, cdr)))
    }

    pub fn car(&self) -> Option<Value> {
        match *self {
            Value::Cell(ref cell) => Some(cell.0.to_owned()),
            _ => None,
        }
    }

    pub fn cdr(&self) -> Option<Value> {
        match *self {
            Value::Cell(ref cell) => Some(cell.1.to_owned()),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print(f, self)
    }
}

fn print(f: &mut fmt::Formatter, val: &Value) -> fmt::Result {
    match *val {
        Value::Nil => write!(f, "()"),
        Value::Boolean(ref b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
        Value::Integer(ref i) => write!(f, "{}", i),
        Value::Symbol(ref s) => write!(f, "{}", s),
        Value::Cell(ref cell) => {
            try!(write!(f, "("));
            try!(print_cell(f, cell));
            write!(f, ")")
        }
        Value::Primitive(_) => write!(f, "#<subr>"),
        Value::Closure(_, _) => write!(f, "#<closure>"),
        Value::Macro(_, _) => write!(f, "#<macro>"),
        Value::Undefined => write!(f, "#<undefined>"),
    }
}

fn print_cell(f: &mut fmt::Formatter, pair: &Rc<(Value, Value)>) -> fmt::Result {
    try!(print(f, &pair.0));
    match pair.1 {
        Value::Nil => Ok(()),
        Value::Cell(ref cdr) => {
            try!(write!(f, " "));
            print_cell(f, cdr)
        }
        ref v => {
            try!(write!(f, " . "));
            print(f, v)
        }
    }
}

pub fn vec2cons(former: &[Value], last: Value) -> Value {
    if former.is_empty() {
        last
    } else {
        Value::cons(former[0].to_owned(), vec2cons(&former[1..], last))
    }
}

use std::collections::HashMap;
use value::{Value, vec2cons};
use vm::Global;

pub fn define_primitives() -> Global {
    let mut g = HashMap::new();
    g.insert("print".to_owned(), Value::Primitive(print));
    g.insert("undefined".to_owned(), Value::Primitive(undefined));
    g.insert("cons".to_owned(), Value::Primitive(cons));
    g.insert("car".to_owned(), Value::Primitive(car));
    g.insert("cdr".to_owned(), Value::Primitive(cdr));
    g.insert("eq?".to_owned(), Value::Primitive(eq_p));
    g.insert("pair?".to_owned(), Value::Primitive(pair_p));
    g.insert("not".to_owned(), Value::Primitive(not));
    g.insert("null?".to_owned(), Value::Primitive(null_p));
    g.insert("list".to_owned(), Value::Primitive(list));
    g.insert("+".to_owned(), Value::Primitive(add));
    g.insert("-".to_owned(), Value::Primitive(sub));
    g.insert("*".to_owned(), Value::Primitive(mul));
    g.insert("=".to_owned(), Value::Primitive(eq));
    g.insert(">".to_owned(), Value::Primitive(gt));
    g.insert(">=".to_owned(), Value::Primitive(ge));
    g.insert("<".to_owned(), Value::Primitive(lt));
    g.insert("<=".to_owned(), Value::Primitive(le));
    g
}

#[allow(unknown_lints, needless_pass_by_value)]
fn print(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        for v in args {
            print!("{}", v)
        }
    }
    println!();
    Ok(Value::Undefined)
}

#[allow(unknown_lints, needless_pass_by_value)]
fn undefined(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("wrong number of arguments: undefined".to_owned());
    }
    Ok(Value::Undefined)
}

#[allow(unknown_lints, needless_pass_by_value)]
fn cons(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("wrong number of arguments: cons".to_owned());
    }
    Ok(Value::cons(args[0].to_owned(), args[1].to_owned()))
}

#[allow(unknown_lints, needless_pass_by_value)]
fn car(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("wrong number of arguments: car".to_owned());
    }
    args[0]
        .car()
        .ok_or_else(|| "pair required: car".to_owned())
}

#[allow(unknown_lints, needless_pass_by_value)]
fn cdr(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("wrong number of arguments: cdr".to_owned());
    }
    args[0]
        .cdr()
        .ok_or_else(|| "pair required: cdr".to_owned())
}

// PartialEqをちゃんとしていないので正確なeq?ではない
#[allow(unknown_lints, needless_pass_by_value)]
fn eq_p(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("wrong number of arguments: eq?".to_owned());
    }
    Ok(Value::Boolean(args[0] == args[1]))
}

#[allow(unknown_lints, needless_pass_by_value)]
fn pair_p(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("wrong number of arguments: pair?".to_owned());
    }
    match args[0] {
        Value::Cell(_) => Ok(Value::Boolean(true)),
        _ => Ok(Value::Boolean(false)),
    }
}

#[allow(unknown_lints, needless_pass_by_value)]
fn not(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("wrong number of arguments: not".to_owned());
    }
    match args[0] {
        Value::Boolean(false) => Ok(Value::Boolean(true)),
        _ => Ok(Value::Boolean(false)),
    }
}

#[allow(unknown_lints, needless_pass_by_value)]
fn null_p(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("wrong number of arguments: null?".to_owned());
    }
    match args[0] {
        Value::Nil => Ok(Value::Boolean(true)),
        _ => Ok(Value::Boolean(false)),
    }
}

#[allow(unknown_lints, needless_pass_by_value)]
fn list(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        Ok(Value::Nil)
    } else {
        Ok(vec2cons(&args, Value::Nil))
    }
}

#[allow(unknown_lints, needless_pass_by_value)]
fn add(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        Ok(Value::Integer(0))
    } else {
        fold_numeric_op(&args, |x, y| x + y).ok_or_else(|| {
                                                            "all arguments must be integers: +"
                                                                .to_owned()
                                                        })
    }
}

#[allow(unknown_lints, needless_pass_by_value)]
fn sub(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        Err("wrong number of arguments: -".to_owned())
    } else if args.len() == 1 {
        match args[0] {
            Value::Integer(num) => Ok(Value::Integer(-num)),
            _ => Err("all arguments must be integers: -".to_owned()),
        }
    } else {
        fold_numeric_op(&args, |x, y| x - y).ok_or_else(|| {
                                                            "all arguments must be integers: -"
                                                                .to_owned()
                                                        })
    }
}

#[allow(unknown_lints, needless_pass_by_value)]
fn mul(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        Ok(Value::Integer(1))
    } else {
        fold_numeric_op(&args, |x, y| x + y).ok_or_else(|| {
                                                            "all arguments must be integers: *"
                                                                .to_owned()
                                                        })
    }
}

fn fold_numeric_op<F>(args: &[Value], f: F) -> Option<Value>
    where F: Fn(i32, i32) -> i32
{
    if let Value::Integer(mut acc) = args[0] {
        for i in args[1..].iter() {
            if let Value::Integer(num) = *i {
                acc = f(acc, num);
            } else {
                return None;
            }
        }
        Some(Value::Integer(acc))
    } else {
        None
    }
}

#[allow(unknown_lints, needless_pass_by_value)]
fn eq(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("wrong number of arguments: =".to_owned());
    }
    fold_numeric_ord(&args, |x, y| x == y).ok_or_else(|| {
                                                          "all arguments must be integers: ="
                                                              .to_owned()
                                                      })
}

#[allow(unknown_lints, needless_pass_by_value)]
fn gt(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("wrong number of arguments: >".to_owned());
    }
    fold_numeric_ord(&args, |x, y| x > y).ok_or_else(|| {
                                                         "all arguments must be integers: >"
                                                             .to_owned()
                                                     })
}

#[allow(unknown_lints, needless_pass_by_value)]
fn ge(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("wrong number of arguments: >=".to_owned());
    }
    fold_numeric_ord(&args, |x, y| x >= y).ok_or_else(|| {
                                                          "all arguments must be integers: >="
                                                              .to_owned()
                                                      })
}

#[allow(unknown_lints, needless_pass_by_value)]
fn lt(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("wrong number of arguments: <".to_owned());
    }
    fold_numeric_ord(&args, |x, y| x < y).ok_or_else(|| {
                                                         "all arguments must be integers: <"
                                                             .to_owned()
                                                     })
}

#[allow(unknown_lints, needless_pass_by_value)]
fn le(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("wrong number of arguments: <=".to_owned());
    }
    fold_numeric_ord(&args, |x, y| x <= y).ok_or_else(|| {
                                                          "all arguments must be integers:: <="
                                                              .to_owned()
                                                      })
}

fn fold_numeric_ord<F>(args: &[Value], f: F) -> Option<Value>
    where F: Fn(i32, i32) -> bool
{
    if let Value::Integer(mut current) = args[0] {
        for i in args[1..].iter() {
            if let Value::Integer(next) = *i {
                if f(current, next) {
                    current = next
                } else {
                    return Some(Value::Boolean(false));
                }
            } else {
                return None;
            }
        }
        Some(Value::Boolean(true))
    } else {
        None
    }
}

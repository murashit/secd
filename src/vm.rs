use std::collections::HashMap;
use std::fmt;
use compiler::Ast;
use value::{Value, vec2cons};

pub type Global = HashMap<String, Value>;

pub struct Machine {
    stack: Stack,
    env: Env,
    code: Code,
    dump: Dump,
}

type Stack = Vec<Value>;
pub type Code = Vec<CodeOp>;
pub type Env = Vec<Vec<Value>>;
type Dump = Vec<DumpOp>;

#[derive(Debug, Clone, PartialEq)]
pub enum CodeOp {
    Ld(Location),
    Ldc(Ast),
    Ldg(String),
    Ldf(Code),
    App(usize),
    Rtn,
    Sel(Code, Code),
    Join,
    Def(String),
    Defm(String),
    Pop,
}

pub type Location = (usize, Position);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Index(usize),
    Rest(usize),
}

#[derive(Debug, Clone, PartialEq)]
enum DumpOp {
    DumpApp(Stack, Env, Code),
    DumpSel(Code),
}

impl Machine {
    pub fn run(env: Env, code: Code, global: &mut Global) -> Result<Value, String> {
        let mut machine = Machine {
            stack: Vec::new(),
            env: env,
            code: code,
            dump: Vec::new(),
        };
        loop {
            if let Some(op) = machine.code.pop() {
                machine.tick(op, global)?;
            } else {
                match machine.stack.pop() {
                    Some(v) => return Ok(v),
                    None => return Ok(Value::Undefined),
                }
            }
        }
    }

    fn tick(&mut self, op: CodeOp, global: &mut Global) -> Result<(), String> {
        match op {
            CodeOp::Ld(location) => {
                let value = get_var(&self.env, location)
                    .ok_or_else(|| "Runtime error: Ld")?;
                self.stack.push(value);
                Ok(())
            }
            CodeOp::Ldc(ast) => Ok(self.stack.push(ast.to_value())),
            CodeOp::Ldg(name) => {
                let value = global
                    .get(&name)
                    .ok_or_else(|| format!("unbound variable: {}", name))?;
                self.stack.push(value.to_owned());
                Ok(())
            }
            CodeOp::Ldf(code) => {
                self.stack
                    .push(Value::Closure(code, self.env.to_owned()));
                Ok(())
            }
            CodeOp::App(i) => {
                let n = self.stack.len() - 1;
                if i > n {
                    return Err("Runtime error: App".to_owned());
                }
                match self.stack.pop() {
                    Some(Value::Closure(code, mut env)) => {
                        env.push(self.stack.drain(n - i..).collect());
                        self.dump
                            .push(DumpOp::DumpApp(self.stack.to_owned(),
                                                  self.env.to_owned(),
                                                  self.code.to_owned()));
                        self.stack.clear();
                        self.env = env;
                        self.code = code;
                        Ok(())
                    }
                    Some(Value::Primitive(procedure)) => {
                        let result = (procedure)(self.stack.drain(n - i..).collect())?;
                        self.stack.push(result);
                        Ok(())
                    }
                    _ => Err("Runtime error: App".to_owned()),
                }
            }
            CodeOp::Rtn => {
                if let (Some(s), Some(DumpOp::DumpApp(mut stack, env, code))) =
                    (self.stack.pop(), self.dump.pop()) {
                    stack.push(s);
                    self.stack = stack;
                    self.env = env;
                    self.code = code;
                    Ok(())
                } else {
                    Err("Runtime error: Rtn".to_owned())
                }
            }
            CodeOp::Sel(conseq, alt) => {
                let value = self.stack.pop().ok_or_else(|| "Runtime error: Sel")?;
                self.dump.push(DumpOp::DumpSel(self.code.to_owned()));
                if value == Value::Boolean(false) {
                    self.code = alt;
                } else {
                    self.code = conseq;
                }
                Ok(())
            }
            CodeOp::Join => {
                if let Some(DumpOp::DumpSel(code)) = self.dump.pop() {
                    self.code = code;
                    Ok(())
                } else {
                    Err("Runtime error: Join".to_owned())
                }
            }
            CodeOp::Def(name) => {
                let value = self.stack.pop().ok_or_else(|| "Runtime error: Def")?;
                global.insert(name, value);
                Ok(())
            }
            CodeOp::Defm(name) => {
                if let Some(Value::Closure(code, env)) = self.stack.pop() {
                    global.insert(name, Value::Macro(code, env));
                    Ok(())
                } else {
                    unimplemented!()
                }
            }
            CodeOp::Pop => {
                self.stack.pop();
                Ok(())
            }
        }
    }
}

impl fmt::Debug for Machine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Machine:")?;
        writeln!(f, "  stack: {:?}", self.stack)?;
        writeln!(f, "  env:   {:?}", self.env)?;
        writeln!(f, "  code:  {:?}", self.code)?;
        writeln!(f, "  dump:  {:?}", self.dump)?;
        write!(f, "")
    }
}

fn get_var(env: &[Vec<Value>], location: Location) -> Option<Value> {
    let (i, j) = (location.0, location.1);
    if let Some(frame) = env.get(i) {
        match j {
            Position::Index(index) => {
                if index < frame.len() {
                    Some(frame[index].to_owned())
                } else {
                    None
                }
            }
            Position::Rest(index) => {
                if index <= frame.len() {
                    let (_, rest) = frame.split_at(index);
                    Some(vec2cons(rest, Value::Nil))
                } else {
                    None
                }
            }
        }
    } else {
        None
    }
}

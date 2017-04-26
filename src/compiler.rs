use value::{Value, vec2cons};
use vm::{Machine, Code, Global, CodeOp, Location, Position};

#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
    Nil,
    Boolean(bool),
    Integer(i32),
    Symbol(String),
    List(Vec<Ast>, Box<Ast>),
    Undefined,
}

type Env = Vec<Ast>;

impl Ast {
    pub fn to_value(&self) -> Value {
        match *self {
            Ast::Nil => Value::Nil,
            Ast::Boolean(b) => Value::Boolean(b),
            Ast::Integer(i) => Value::Integer(i),
            Ast::Symbol(ref s) => Value::Symbol(s.to_owned()),
            Ast::List(ref former, ref last) => {
                vec2cons(&former
                              .iter()
                              .map(|x| x.to_value())
                              .collect::<Vec<Value>>(),
                         last.to_value())
            }
            Ast::Undefined => Value::Undefined,
        }
    }

    pub fn new_symbol(name: &str) -> Ast {
        Ast::Symbol(name.to_owned())
    }

    pub fn new_list(former: &[Ast], last: Ast) -> Ast {
        Ast::List(former.to_owned(), Box::new(last))
    }

    pub fn compile(&self, global: &Global) -> Result<Code, String> {
        let mut env = Vec::new();
        let mut code = Vec::new();
        self.compile_helper(&mut env, &mut code, global)?;
        Ok(code)
    }

    fn compile_helper(&self,
                      env: &mut Env,
                      code: &mut Code,
                      global: &Global)
                      -> Result<(), String> {
        match *self {
            Ast::Symbol(ref name) => {
                if let Some(location) = location(self, env) {
                    code.push(CodeOp::Ld(location));
                } else {
                    code.push(CodeOp::Ldg(name.to_owned()));
                }
                Ok(())
            }
            Ast::List(ref form, ref last) => {
                if **last != Ast::Nil {
                    return Err("proper list required".to_owned());
                }
                if let Some(&Ast::Symbol(ref name)) = form.get(0) {
                    if let Some(&Value::Macro(ref macro_code, _)) = global.get(name) {
                        let mut macro_code = macro_code.to_owned();
                        macro_code.remove(0); // 最後のRtnを削除
                        let macro_args = form[1..].iter().map(|ast| ast.to_value()).collect();
                        let result =
                            Machine::run(vec![macro_args], macro_code, &mut global.to_owned());
                        result
                            .unwrap()
                            .to_ast()
                            .compile_helper(env, code, global)
                    } else {
                        match name.as_str() {
                            "quote" => {
                                if form.len() != 2 {
                                    return Err("malformed quote".to_owned());
                                }
                                code.push(CodeOp::Ldc(form[1].to_owned()));
                                Ok(())
                            }
                            "define" => {
                                if form.len() < 3 {
                                    return Err("malformed define".to_owned());
                                }
                                define(&form[1], &form[2..], env, code, global)
                            }
                            "define-macro" => {
                                if form.len() < 3 {
                                    return Err("malformed define-macro".to_owned());
                                }
                                define_macro(&form[1], &form[2..], env, code, global)
                            }
                            "lambda" => {
                                let (params, body) = form[1..].split_at(1);
                                lambda(params[0].to_owned(), body, env, code, global)
                            }
                            "if" => {
                                let n = form.len();
                                if n < 3 || 4 < n {
                                    return Err("malformed if".to_owned());
                                }
                                let alt = form.get(3);
                                if_(&form[1], &form[2], alt, env, code, global)
                            }
                            "begin" => {
                                if form.len() < 2 {
                                    code.push(CodeOp::Ldc(Ast::Integer(0)));
                                    Ok(())
                                } else {
                                    begin(&form[1..], env, code, global)
                                }
                            }
                            _ => apply(form, env, code, global),
                        }
                    }
                } else {
                    apply(form, env, code, global)
                }
            }
            ref ast => {
                code.push(CodeOp::Ldc(ast.to_owned()));
                Ok(())
            }
        }
    }
}

fn begin(body: &[Ast], env: &mut Env, code: &mut Code, global: &Global) -> Result<(), String> {
    for exp in body.iter().rev() {
        exp.compile_helper(env, code, global)?;
        code.push(CodeOp::Pop)
    }
    code.pop();
    Ok(())
}

fn lambda(params: Ast,
          body: &[Ast],
          env: &mut Env,
          code: &mut Code,
          global: &Global)
          -> Result<(), String> {
    let mut new_env = env;
    new_env.push(params);
    let mut body_code = vec![CodeOp::Rtn];
    begin(body, &mut new_env, &mut body_code, global)?;
    code.push(CodeOp::Ldf(body_code));
    Ok(())
}

fn if_(pred: &Ast,
       conseq: &Ast,
       alt: Option<&Ast>,
       env: &mut Env,
       code: &mut Code,
       global: &Global)
       -> Result<(), String> {
    let mut conseq_code = vec![CodeOp::Join];
    conseq.compile_helper(env, &mut conseq_code, global)?;
    let mut alt_code = vec![CodeOp::Join];
    alt.unwrap_or(&Ast::Undefined)
        .compile_helper(env, &mut alt_code, global)?;
    code.push(CodeOp::Sel(conseq_code, alt_code));
    pred.compile_helper(env, code, global)?;
    Ok(())
}

fn apply(form: &[Ast], env: &mut Env, code: &mut Code, global: &Global) -> Result<(), String> {
    code.push(CodeOp::App(form[1..].len()));
    form[0].compile_helper(env, code, global)?;
    for ast in form[1..].iter().rev() {
        ast.compile_helper(env, code, global)?;
    }
    Ok(())
}

fn define(head: &Ast,
          tail: &[Ast],
          env: &mut Env,
          code: &mut Code,
          global: &Global)
          -> Result<(), String> {
    match *head {
        Ast::Symbol(ref name) => {
            if tail.len() != 1 {
                return Err("malformed define".to_owned());
            }
            code.push(CodeOp::Def(name.to_owned()));
            tail[0].compile_helper(env, code, global)?;
            Ok(())
        }
        Ast::List(ref former, ref last) => {
            if let Some(&Ast::Symbol(ref name)) = former.get(0) {
                code.push(CodeOp::Def(name.to_owned()));
                let params = Ast::new_list(&former[1..], *last.to_owned());
                lambda(params, tail, env, code, global)
            } else {
                Err("malformed define".to_owned())
            }
        }
        _ => Err("malformed define".to_owned()),
    }
}

fn define_macro(head: &Ast,
                tail: &[Ast],
                env: &mut Env,
                code: &mut Code,
                global: &Global)
                -> Result<(), String> {
    match *head {
        Ast::Symbol(ref name) => {
            if tail.len() != 1 {
                return Err("malformed define-macro".to_owned());
            }
            code.push(CodeOp::Defm(name.to_owned()));
            tail[0].compile_helper(env, code, global)?;
            Ok(())
        }
        Ast::List(ref former, ref last) => {
            if let Some(&Ast::Symbol(ref name)) = former.get(0) {
                code.push(CodeOp::Defm(name.to_owned()));
                let params = Ast::new_list(&former[1..], *last.to_owned());
                lambda(params, tail, env, code, global)
            } else {
                Err("malformed define-macro".to_owned())
            }
        }
        _ => Err("malformed define-macro".to_owned()),
    }
}

fn location(sym: &Ast, env: &[Ast]) -> Option<Location> {
    for (i, frame) in env.iter().enumerate() {
        if let Some(j) = position(sym, frame) {
            return Some((i, j));
        }
    }
    None
}

fn position(sym: &Ast, frame: &Ast) -> Option<Position> {
    match *frame {
        Ast::List(ref vec, ref last) => {
            if let Some(i) = vec.iter().position(|x| sym == x) {
                Some(Position::Index(i))
            } else if *sym == **last {
                Some(Position::Rest(vec.len()))
            } else {
                None
            }
        }
        Ast::Symbol(_) => {
            if sym == frame {
                Some(Position::Rest(0))
            } else {
                None
            }
        }
        _ => None,
    }
}

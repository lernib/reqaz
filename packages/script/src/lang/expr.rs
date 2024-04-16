use crate::runtime::{Processable, Runtime, Value};
use super::{Parse, ParseError};
use super::Lit;


pub enum Expr {
    Call(ExprCall),
    Ident(String),
    Lit(Lit)
}

impl Expr {
    pub fn new_call(func: Box<Expr>, args: Vec<Expr>) -> Self {
        Expr::Call(ExprCall { func, args })
    }
}

impl Parse for Expr {
    fn parse(s: &str) -> Result<Self, ParseError> {
        crate::parse::ExprParser::new().parse(s)
    }
}

impl Processable for Expr {
    fn process(&self, runtime: &mut Runtime) -> Option<Value> {
        match self {
            Expr::Call(ec) => ec.process(runtime),
            Expr::Ident(ei) => runtime.ctx().get(ei).cloned(),
            Expr::Lit(el) => el.process(runtime)
        }
    }
}

pub struct ExprCall {
    pub func: Box<Expr>,
    pub args: Vec<Expr>
}

impl Processable for ExprCall {
    fn process(&self, runtime: &mut Runtime) -> Option<Value> {
        // every function is log lol
        let args = self.args
            .iter()
            .map(|a| a.process(runtime))
            .filter_map(|v| v)
            .collect::<Vec<_>>();

        for arg in &args {
            print!("{}", arg.to_string());
        }

        if args.is_empty() {
            println!("<no args>")
        } else {
            println!("");
        }

        None
    }
}

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

pub struct ExprCall {
    pub func: Box<Expr>,
    pub args: Vec<Expr>
}

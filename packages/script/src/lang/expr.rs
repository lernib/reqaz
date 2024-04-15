use super::{Parse, ParseError};
use super::Lit;


pub enum Expr {
    Lit(Lit)
}

impl Parse for Expr {
    fn parse(s: &str) -> Result<Self, ParseError> {
        crate::parse::ExprParser::new().parse(s)
    }
}

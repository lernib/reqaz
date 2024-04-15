use super::{Parse, ParseError};


pub enum Lit {
    Int(i64)
}

impl Parse for Lit {
    fn parse(s: &str) -> Result<Self, ParseError> {
        crate::parse::LitParser::new().parse(s)
    }
}

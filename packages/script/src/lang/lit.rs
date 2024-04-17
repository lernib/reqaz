use crate::runtime::{Processable, Runtime, Value};
use super::{Parse, ParseError};


pub enum Lit {
    Int(i64),
    Str(String),
    Bool(bool)
}

impl Parse for Lit {
    fn parse(s: &str) -> Result<Self, ParseError> {
        crate::parse::LitParser::new().parse(s)
    }
}

impl Processable for Lit {
    fn process(&self, _runtime: &mut Runtime) -> Option<Value> {
        match self {
            Lit::Int(i) => Some(Value::Number(*i)),
            Lit::Str(s) => Some(Value::String(s.to_string())),
            Lit::Bool(b) => Some(Value::Boolean(*b))
        }
    }
}

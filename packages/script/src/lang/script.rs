use super::{Parse, ParseError};
use super::Item;


pub struct Script {
    items: Vec<Item>
}

impl Parse for Script {
    fn parse(s: &str) -> Result<Self, ParseError> {
        crate::parse::ScriptParser::new().parse(s)
    }
}

impl From<Vec<Item>> for Script {
    fn from(items: Vec<Item>) -> Self {
        Script { items }
    }
}

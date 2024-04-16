use crate::runtime::{Processable, Runtime, Value};
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

impl Processable for Script {
    fn process(&self, runtime: &mut Runtime) -> Option<Value> {
        for item in self.items.iter() {
            item.process(runtime);
        }

        None
    }
}

impl From<Vec<Item>> for Script {
    fn from(items: Vec<Item>) -> Self {
        Script { items }
    }
}

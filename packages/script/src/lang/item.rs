use crate::runtime::{Processable, Runtime, Value};
use super::{Parse, ParseError};
use super::Expr;


pub enum Item {
    Let(ItemLet),
    Expr(Expr)
}

impl Item {
    pub fn new_let(ident: String, expr: Expr) -> Self {
        Item::Let(ItemLet { ident, expr })
    }
}

impl Parse for Item {
    fn parse(s: &str) -> Result<Self, ParseError> {
        crate::parse::ItemParser::new().parse(s)
    }
}

impl Processable for Item {
    fn process(&self, runtime: &mut Runtime) -> Option<Value> {
        match self {
            Item::Let(il) => il.process(runtime),
            Item::Expr(e) => e.process(runtime)
        }
    }
}

pub struct ItemLet {
    pub ident: String,
    pub expr: Expr
}

impl Processable for ItemLet {
    fn process(&self, runtime: &mut Runtime) -> Option<Value> {
        let val = self.expr.process(runtime)?;

        runtime.ctx_mut().set(self.ident.clone(), val);

        None
    }
}

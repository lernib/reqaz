use crate::runtime::{Processable, Runtime, Value};
use super::Expr;


pub struct ExprBinary {
    left: Box<Expr>,
    right: Box<Expr>,
    op: BinOp
}

impl ExprBinary {
    pub fn new(left: Expr, op: BinOp, right: Expr) -> Self {
        ExprBinary {
            left: Box::new(left),
            right: Box::new(right),
            op
        }
    }
}

impl Processable for ExprBinary {
    fn process(&self, runtime: &mut Runtime) -> Option<Value> {
        let left = self.left.process(runtime)?;
        let right = self.right.process(runtime)?;

        match self.op {
            BinOp::Add => left + right,
            BinOp::Sub => left - right,
            BinOp::Mul => left * right,
            BinOp::Div => left / right,
        }
    }
}

pub enum BinOp {
    Add, Sub,
    Mul, Div
}

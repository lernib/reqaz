use std::ops::{Add, Div, Mul, Sub};


#[derive(Clone)]
pub enum Value {
    Number(i64),
    String(String),
    Boolean(bool),
    Function(FunctionValue)
}

impl Value {
    pub fn new_function(f: fn(Vec<Value>) -> Option<Value>) -> Value {
        Value::Function(FunctionValue::Rust(f))
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.into(),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Function(_) => "<function>".into()
        }
    }
}

macro_rules! ident_variant {
    (number -> $v:pat) => { Value::Number($v) };
    (string -> $v:pat) => { Value::String($v) };
    (boolean -> $v:pat) => { Value::Boolean($v) }
}

macro_rules! operator_impl {
    (
        [$lhs:expr, $rhs:expr];
        $(
            ( $t1:ident -> $v1:ident, $t2:ident -> $v2:ident ) => $e:expr
        ),*
    ) => {
        match ($lhs, $rhs) {
            $(
                (
                    ident_variant!($t1 -> $v1),
                    ident_variant!($t2 -> $v2)
                ) => $e,
            )*
            _ => None
        }
    }
}

impl Add<Value> for Value {
    type Output = Option<Value>;

    fn add(self, rhs: Value) -> Self::Output {
        operator_impl! {
            [self, rhs];
            (number -> n1, number -> n2) => Some(Value::Number(n1 + n2)),
            (string -> s1, string -> s2) => Some(Value::String(s1 + &s2))
        }
    }
}

impl Sub<Value> for Value {
    type Output = Option<Value>;

    fn sub(self, rhs: Value) -> Self::Output {
        operator_impl! {
            [self, rhs];
            (number -> n1, number -> n2) => Some(Value::Number(n1 - n2))
        }
    }
}

impl Mul<Value> for Value {
    type Output = Option<Value>;

    fn mul(self, rhs: Value) -> Self::Output {
        operator_impl! {
            [self, rhs];
            (number -> n1, number -> n2) => Some(Value::Number(n1 * n2))
        }
    }
}

impl Div<Value> for Value {
    type Output = Option<Value>;

    fn div(self, rhs: Value) -> Self::Output {
        operator_impl! {
            [self, rhs];
            (number -> n1, number -> n2) => Some(Value::Number(n1 / n2))
        }
    }
}

#[derive(Clone)]
pub enum FunctionValue {
    Rust(fn(Vec<Value>) -> Option<Value>)
}

impl FunctionValue {
    pub fn call(&self, args: Vec<Value>) -> Option<Value> {
        match self {
            FunctionValue::Rust(func) => func(args)
        }
    }
}

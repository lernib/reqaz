use std::collections::HashMap;

pub mod val;

pub use val::Value;


#[derive(Default)]
pub struct Runtime<'me> {
    ctx: Context<'me>
}

impl<'me> Runtime<'me> {
    pub fn process<P: Processable>(&mut self, processable: P) -> Option<Value> {
        processable.process(self)
    }

    pub fn ctx(&self) -> &Context<'me> {
        &self.ctx
    }

    pub fn ctx_mut(&mut self) -> &mut Context<'me> {
        &mut self.ctx
    }
}

pub trait Processable {
    fn process(&self, runtime: &mut Runtime) -> Option<Value>;
}

#[derive(Default)]
pub struct Context<'me> {
    parent: Option<&'me Context<'me>>,
    map: HashMap<String, Value>
}

impl<'me> Context<'me> {
    pub fn new(parent: Option<&'me Context<'me>>) -> Self {
        Context {
            parent,
            map: Default::default()
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(key).or_else(|| self.parent?.get(key))
    }

    pub fn set(&mut self, key: String, value: Value) -> Option<Value> {
        self.map.insert(key, value)
    }
}

use lalrpop_util::lexer::Token;

mod expr;
mod item;
mod lit;
mod op;
mod script;

pub use expr::*;
pub use item::*;
pub use lit::*;
pub use op::*;
pub use script::*;


type ParseError<'input> = lalrpop_util::ParseError<usize, Token<'input>, &'static str>;

pub trait Parse: Sized {
    fn parse<'input>(s: &'input str) -> Result<Self, ParseError<'input>>;
}

use lalrpop_util::lexer::Token;

mod expr;
mod item;
mod lit;
mod script;

pub use expr::Expr;
pub use item::Item;
pub use lit::Lit;
pub use script::Script;


type ParseError<'input> = lalrpop_util::ParseError<usize, Token<'input>, &'static str>;

pub trait Parse: Sized {
    fn parse<'input>(s: &'input str) -> Result<Self, ParseError<'input>>;
}

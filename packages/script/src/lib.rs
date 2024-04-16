use lalrpop_util::lalrpop_mod;

pub mod lang;
pub mod runtime;

lalrpop_mod!(pub parse);


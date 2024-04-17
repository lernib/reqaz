use lalrpop_util::lalrpop_mod;

pub mod lang;
pub mod runtime;

lalrpop_mod!(pub parse);

pub fn apply_string_escapes(content: &str) -> std::borrow::Cow<str> {
    if !content.contains('\\') {
        content.into()
    } else {
        let mut iter = content.chars();
        let mut text = String::new();
        while let Some(c) = iter.next() {
            let the_char = if c != '\\' {
                c
            } else {
                let next = iter.next().unwrap();
                match next {
                    '\\' | '\"' => next,
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => panic!("unrecognized escape: \\{}", next),
                }
            };
            text.push(the_char);
        }
        text.into()
    }
}

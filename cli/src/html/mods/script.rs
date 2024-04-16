use eyre::eyre;
use eyre::Result;
use html5ever::{ns, namespace_url};
use html5ever::{QualName, LocalName};
use hyper::Uri;
use lazy_static::lazy_static;
use nib_script::lang::Parse;
use nib_script::lang::Script;
use nib_script::runtime::Processable;
use nib_script::runtime::Runtime;
use super::HtmlMod;
use std::sync::OnceLock;


lazy_static! {
    static ref QUAL_NAME: QualName = QualName::new(
        None,
        ns!(html),
        LocalName::from("nib:script")
    );
}

static RUNTIME_STD: OnceLock<Runtime> = OnceLock::new();

fn runtime_std() -> Runtime<'static> {
    RUNTIME_STD.get_or_init(|| {
        let mut runtime = Runtime::default();

        runtime.register_function("log", |args| {
            for arg in &args {
                print!("{}", arg.to_string());
            }
    
            if args.is_empty() {
                println!("<no args>")
            } else {
                println!("");
            }

            None
        });

        runtime
    }).clone()
}

pub struct ScriptMod {
    page_uri: Uri
}

impl ScriptMod {
    pub fn new(page_uri: Uri) -> Self {
        ScriptMod {
            page_uri
        }
    }
}

impl HtmlMod for ScriptMod {
    fn modify(&self, html: super::Html) -> Result<super::Html> {
        let mut runtime = runtime_std();

        for node in html.descendants() {
            if let Some(el) = node.as_element() {
                if el.name == *QUAL_NAME {
                    let contents = node.text_contents();

                    let script = Script::parse(&contents)
                        .map_err(|e| eyre!("Invalid script: {}", e))?;

                    script.process(&mut runtime);

                    // detach script
                    node.detach()
                }
            }
        }

        Ok(html)
    }
}

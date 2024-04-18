use crate::html::attr::*;
use color_eyre::owo_colors::OwoColorize;
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
use nib_script::runtime::Value;
use super::fetch::*;
use super::Html;
use super::HtmlMod;


lazy_static! {
    static ref QUAL_NAME: QualName = QualName::new(
        None,
        ns!(html),
        LocalName::from("nib:script")
    );
}

fn runtime_std<'r>() -> Runtime<'r> {
    let mut runtime = Runtime::default();

    let nibscript_data = ScriptModData::default();

    runtime.register_value(
        "__nibscript_data",
        Value::new_rsdata_rc_refcell(nibscript_data)
    );

    runtime.register_function("fetch_tag", |runtime, args| {
        let href = args.first()?.to_string();
        let href = Href::try_from(href.as_str()).ok()?;

        let is_component = match runtime.ctx().get("FETCH_COMPONENT") {
            Some(Value::Boolean(b)) => *b,
            _ => false
        };

        let node = runtime.ctx().get("__nibscript_node")?
            .as_rc_refcell()?;
        let node = node
            .try_borrow()
            .ok()?;
        let node = node
            .downcast_ref::<Html>()?
            .clone();

        let binding = runtime.ctx().get("__nibscript_data")?
            .as_rc_refcell()?;
        let mut binding = binding
            .try_borrow_mut()
            .ok()?;
        let nibscript_data = binding
            .downcast_mut::<ScriptModData>()?;

        nibscript_data.tag_fetches.push(TagFetch {
            href,
            node,
            component: is_component
        });

        None
    });

    runtime.register_function("log", |_, args| {
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
    fn modify(&self, html: Html) -> Result<Html> {
        let mut runtime = runtime_std();
        let mut script_nodes = vec![];

        for node in html.descendants() {
            if let Some(el) = node.as_element() {
                if el.name == *QUAL_NAME {
                    runtime.register_value(
                        "__nibscript_node",
                        Value::new_rsdata_rc_refcell(node.clone())
                    );

                    let contents = node.text_contents();

                    let script = Script::parse(&contents)
                        .map_err(|e| eyre!("Invalid script: {}", e))?;

                    script.process(&mut runtime);

                    script_nodes.push(node);
                }
            }
        }

        // get the data
        let nibscript_data = runtime.ctx().get("__nibscript_data")
            .ok_or(eyre!("[YOUR FAULT] __nibscript_data replaced"))?
            .as_rc_refcell()
            .ok_or(eyre!("[YOUR FAULT] __nibscript_data replaced"))?;
        let nibscript_data = nibscript_data
            .try_borrow()?;
        let nibscript_data = nibscript_data
            .downcast_ref::<ScriptModData>()
            .ok_or(eyre!("[MY FAULT] Downcast was not proper type"))?
            .clone();

        let mut tag_fetches = nibscript_data.tag_fetches;
        tag_fetches.reverse();

        for fetch in tag_fetches {
            let type_ = match fetch.component {
                false => None,
                true => Some("component".into())
            };

            let el = perform_fetch(fetch.href, &self.page_uri, type_)?;

            fetch.node.insert_after(el);
        }

        for node in script_nodes {
            node.detach()
        }

        Ok(html)
    }
}

#[derive(Default, Clone)]
struct ScriptModData {
    tag_fetches: Vec<TagFetch>
}

#[derive(Clone)]
struct TagFetch {
    href: Href,
    node: Html,
    component: bool
}

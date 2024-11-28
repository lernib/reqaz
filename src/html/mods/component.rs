use super::fetch::InsertResponseError;
use super::Html;
use super::HtmlMod;
use super::HtmlModManager;
use crate::html::attr::{GetAttr, Href};
use crate::mediatype::TEXT_HTML;
use crate::source::{ResolverError, SourceResolver};
use html5ever::QualName;
use html5ever::{local_name, namespace_url, ns};
use html_escape::decode_html_entities;
use http::uri::InvalidUriParts;
use http::Uri;
use kuchikiki::traits::TendrilSink;
use kuchikiki::ElementData;
use kuchikiki::NodeData::DocumentFragment;
use kuchikiki::NodeRef;
use mediatype::MediaTypeBuf;
use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::io::Error as IoError;

/// The component reqaz HTML mods
///
/// This mod handles any modular components in a webpage.
pub struct Mod {
    page_uri: Uri,

    /// A source resolver for internal fetches
    resolver: SourceResolver,
}

impl Mod {
    #[allow(clippy::single_call_fn)]
    /// Create a new Component mod instance
    pub const fn new(page_uri: Uri, resolver: SourceResolver) -> Self {
        Self { page_uri, resolver }
    }

    /// Fetch an HTML node for an href and return it
    fn perform_fetch(&self, href: Href) -> Result<Html, ComponentModError> {
        if href.extension() != Some("html".to_string()) {
            return Err(ComponentModError::LinkNotHtml);
        }

        let resp = match href {
            Href::Absolute(_) | Href::Relative(_) => href
                .append_to_uri(&self.page_uri)
                .map_err(ComponentModError::InvalidUriParts)
                .and_then(|uri| {
                    self.resolver
                        .resolve_source(&uri)
                        .map_err(ComponentModError::ResolverError)
                        .map(|resolved| resolved.body)
                }),
            Href::Uri(uri) => ureq::get(&uri.to_string())
                .call()
                .map_err(|err| ComponentModError::Network(Box::new(err)))
                .and_then(|resp| {
                    let mime = MediaTypeBuf::from_string(resp.content_type().to_owned()).ok();

                    if mime == Some(TEXT_HTML.into()) {
                        let mut body = vec![];

                        resp.into_reader()
                            .read_to_end(&mut body)
                            .map_err(ComponentModError::IoError)?;

                        Ok(body)
                    } else {
                        Err(ComponentModError::LinkNotHtml)
                    }
                }),
            Href::Other(_) => Err(ComponentModError::InvalidHref(href.clone())),
        }?;

        let contents = String::from_utf8(resp).or(Err(ComponentModError::LinkNotHtml))?;

        // Elements are required to be HTML
        html_from_string(&contents)
    }
}

fn html_from_string(s: &str) -> Result<Html, ComponentModError> {
    let html =
        kuchikiki::parse_fragment(QualName::new(None, ns!(html), local_name!("div")), vec![])
            .one(s);

    let first_child = html
        .first_child()
        .ok_or(InsertResponseError(TEXT_HTML.into()))?;

    let el = NodeRef::new(DocumentFragment);

    for child in first_child.children() {
        el.append(child);
    }

    Ok(el)
}

/// Insert possible props into locations for an HTML segment
fn process_props(html: Html, props: ComponentData) -> Html {
    let mut to_check = html.children().into_iter().collect::<VecDeque<_>>();

    while let Some(to_check_el) = to_check.pop_front() {
        let Some(node_el) = to_check_el.as_element() else {
            continue;
        };

        if &node_el.name.local == "title" {
            // If it's a title, it needs to be handled specially
            // Get the text content and parse, then replace
            let content = to_check_el
                .children()
                .filter_map(|el| {
                    // This filter should literally never be needed but heck it type safety
                    let text_el = el.as_text()?;

                    Some(text_el.borrow().clone())
                })
                .reduce(|cur, nxt| cur + &nxt);

            let Some(text_content) = content else {
                continue;
            };

            let parsed = decode_html_entities(&text_content);

            let Ok(node) = html_from_string(&parsed) else {
                // Not gonna handle failed parsing of title contents bc that's stupid
                continue;
            };

            let node = process_props(node, props.clone());
            let title_text = node.to_string();

            // Replace the title node
            let new_node =
                NodeRef::new_element(QualName::new(None, ns!(html), local_name!("title")), vec![]);

            let title_text_node = NodeRef::new_text(title_text);

            new_node.append(title_text_node);

            to_check_el.insert_after(new_node);
            to_check_el.detach();
        } else if &node_el.name.local == "source" {
            if node_el.get_attr("slot").is_none() {
                continue;
            }

            // Slot, insert slot data
            to_check_el.insert_after(props.slot.clone());
            to_check_el.detach();
        } else if &node_el.name.local == "param" {
            // If it's a param with a name but no value, replace it
            let Some(name) = node_el.get_attr("name") else {
                continue;
            };

            if node_el.get_attr("value").is_some() {
                continue;
            };

            let value = props.props.get(&name).cloned().unwrap_or_default();

            to_check_el.insert_after(NodeRef::new_text(value));
            to_check_el.detach();
        } else {
            // Add children
            for child in to_check_el.children() {
                to_check.push_back(child);
            }
        }
    }

    html
}

fn get_props_from_object(node: &Html) -> ComponentData {
    let props = node
        .children()
        .into_iter()
        .filter_map(|child| {
            let Some(node_el) = child.as_element() else {
                return None;
            };

            if "param" != &node_el.name.local {
                return None;
            };

            let name = node_el.get_attr("name");
            let value = node_el.get_attr("value");

            name.zip(value)
        })
        .collect::<HashMap<_, _>>();

    let slot_children = node.children().into_iter().filter_map(|child| {
        let Some(node_el) = child.as_element() else {
            return Some(child);
        };

        if "param" != &node_el.name.local {
            return Some(child);
        };

        None
    });

    let slot = NodeRef::new(DocumentFragment);
    for child in slot_children {
        slot.append(child)
    }

    ComponentData { props, slot }
}

fn get_props_from_link(node: &ElementData) -> ComponentData {
    let props = node
        .attributes
        .borrow()
        .map
        .keys()
        .into_iter()
        .filter_map(|attr| {
            let local = format!("{}", attr.local);

            if !local.starts_with("nib-prop-") {
                return None;
            };

            let name = local.replace("nib-prop-", "");
            let value = node.get_attr(&local);

            value.map(|v| (name, v))
        })
        .collect::<HashMap<_, _>>();

    ComponentData {
        props,
        slot: NodeRef::new(DocumentFragment),
    }
}

impl HtmlMod for Mod {
    fn modify(&self, html: Html, manager: &HtmlModManager) -> Result<Html, super::Error> {
        let components: Vec<_> = html
            .select(r#"object[nib-mod~="component"]"#)
            .map(|sels| {
                sels.into_iter()
                    .chain(
                        html.select(r#"link[nib-mod~="component"]"#)
                            .map(|sels| {
                                sels.into_iter()
                                    // There is definitely a better way to do this but rn i can't be bothered
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default()
                            .into_iter(),
                    )
                    .collect()
            })
            .unwrap_or_default();

        for component in components {
            let node = component.as_node();
            let Some(node_el) = node.as_element() else {
                continue;
            };

            let data_prop = match node_el.name.local {
                local_name!("object") => "data",
                local_name!("link") => "href",
                _ => unreachable!(),
            };

            let href = node_el
                .get_attr(data_prop)
                .and_then(|href| Href::try_from(href.as_str()).ok());

            let props = match node_el.name.local {
                local_name!("object") => get_props_from_object(node),
                local_name!("link") => get_props_from_link(node_el),
                _ => unreachable!(),
            };

            let new_el = href.map(|url| self.perform_fetch(url)).transpose()?;

            if let Some(new_el) = new_el {
                let new_el = process_props(new_el, props);

                // Apply mods to element
                let new_el = manager.apply_mods(new_el)?;

                node.insert_after(new_el);
                node.detach();
            }
        }

        Ok(html)
    }
}

/// Errors possible due to a fetch mod run
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum ComponentModError {
    /// The href supplied was invalid
    InvalidHref(Href),

    /// There was a problem inserting an element
    Insertion(InsertResponseError),

    /// There was a problem creating a URI at some point
    InvalidUriParts(InvalidUriParts),

    /// There was a networking problem
    ///
    /// Wrapped in a box for size concerns
    Network(Box<ureq::Error>),

    /// There was a resolver error
    ResolverError(ResolverError),

    /// There was an IO problem
    IoError(IoError),

    /// The component was not HTML
    LinkNotHtml,
}

impl From<InsertResponseError> for ComponentModError {
    fn from(value: InsertResponseError) -> Self {
        ComponentModError::Insertion(value)
    }
}

#[allow(clippy::missing_trait_methods)]
#[allow(clippy::absolute_paths)]
impl std::error::Error for ComponentModError {}

#[allow(clippy::absolute_paths)]
#[allow(clippy::pattern_type_mismatch)]
impl Display for ComponentModError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidHref(href) => formatter.write_fmt(format_args!("Invalid href: {href}")),
            Self::Insertion(ire) => ire.fmt(formatter),
            Self::InvalidUriParts(iup) => iup.fmt(formatter),
            Self::Network(neterr) => neterr.fmt(formatter),
            Self::ResolverError(resolver_error) => resolver_error.fmt(formatter),
            Self::IoError(ioerr) => ioerr.fmt(formatter),
            Self::LinkNotHtml => {
                formatter.write_str("The component at the specific link is not valid HTML")
            }
        }
    }
}

#[derive(Clone)]
struct ComponentData {
    pub props: HashMap<String, String>,
    pub slot: Html,
}

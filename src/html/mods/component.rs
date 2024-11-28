use super::fetch::InsertResponseError;
use super::Html;
use super::HtmlMod;
use crate::html::attr::{GetAttr, Href};
use crate::mediatype::TEXT_HTML;
use crate::source::{ResolverError, SourceResolver};
use html5ever::QualName;
use html5ever::{local_name, namespace_url, ns};
use http::uri::InvalidUriParts;
use http::Uri;
use kuchikiki::traits::TendrilSink;
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
        let html =
            kuchikiki::parse_fragment(QualName::new(None, ns!(html), local_name!("div")), vec![])
                .one(contents);

        let first_child = html
            .first_child()
            .ok_or(InsertResponseError(TEXT_HTML.into()))?;

        let el = NodeRef::new(DocumentFragment);

        for child in first_child.children() {
            el.append(child);
        }

        Ok(el)
    }
}

/// Insert possible props into locations for an HTML segment
fn process_props(html: Html, props: HashMap<String, String>) -> Html {
    let mut to_check = html.children().into_iter().collect::<VecDeque<_>>();

    while let Some(to_check_el) = to_check.pop_front() {
        let Some(node_el) = to_check_el.as_element() else {
            continue;
        };

        // If it's a param with a name but no value, replace it
        if &node_el.name.local == "param" {
            let Some(name) = node_el.get_attr("name") else {
                continue;
            };

            if node_el.get_attr("value").is_some() {
                continue;
            };

            let value = props.get(&name).cloned().unwrap_or_default();

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

impl HtmlMod for Mod {
    fn modify(&self, html: Html) -> Result<Html, super::Error> {
        let components: Vec<_> = html
            .select(r#"object[nib-mod~="component"]"#)
            .map(|sels| sels.into_iter().collect())
            .unwrap_or_default();

        for component in components {
            let node = component.as_node();
            let Some(node_el) = node.as_element() else {
                continue;
            };

            let href = node_el
                .get_attr("data")
                .and_then(|href| Href::try_from(href.as_str()).ok());

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

            let new_el = href.map(|url| self.perform_fetch(url)).transpose()?;

            if let Some(new_el) = new_el {
                let new_el = process_props(new_el, props);

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

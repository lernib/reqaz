use super::fetch::InsertResponseError;
use super::Html;
use super::HtmlMod;
use crate::html::attr::{GetAttr, Href};
use crate::mediatype::TEXT_HTML;
use crate::source::{ResolverError, SourceResolver};
use http::uri::InvalidUriParts;
use http::Uri;
use kuchikiki::traits::TendrilSink;
use kuchikiki::NodeData::DocumentFragment;
use kuchikiki::NodeRef;
use mediatype::MediaTypeBuf;
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
        let html = kuchikiki::parse_html().one(contents);

        let el = NodeRef::new(DocumentFragment);

        for child in html.children() {
            el.append(child);
        }

        Ok(el)
    }
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
                .get_attr("href")
                .and_then(|href| Href::try_from(href.as_str()).ok());
            let props = node_el.get_attr("nib-props");

            let new_el = href.map(|url| self.perform_fetch(url)).transpose()?;

            if let Some(new_el) = new_el {
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

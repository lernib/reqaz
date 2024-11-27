use super::Html;
use super::HtmlMod;
use crate::html::attr::{GetAttr, Href};
use crate::mediatype::{APPLICATION_OCTET_STREAM, IMG_SVG_XML, TEXT_CSS, TEXT_HTML};
use crate::source::{ResolverError, SourceResolver};
use core::fmt::Display;
use html5ever::QualName;
use html5ever::{local_name, namespace_url, ns};
use http::uri::InvalidUriParts;
use hyper::Uri;
use kuchikiki::traits::TendrilSink;
use kuchikiki::NodeData::DocumentFragment;
use kuchikiki::NodeRef;
use mediatype::MediaTypeBuf;
use std::io::Error as IoError;

/// The Fetch reqaz HTML mod
pub struct Mod {
    /// The URI of the currently loading asset
    page_uri: Uri,

    /// A source resolver for internal fetches
    resolver: SourceResolver,
}

impl Mod {
    #[allow(clippy::single_call_fn)]
    /// Create a new Fetch mod instance
    pub const fn new(page_uri: Uri, resolver: SourceResolver) -> Self {
        Self { page_uri, resolver }
    }

    /// Fetch an HTML node for an href and return it
    fn perform_fetch(&self, href: Href) -> Result<Html, FetchError> {
        href.extension()
            .and_then(|ext| get_element_from_extension(&ext))
            .ok_or_else(|| FetchError::InvalidHref(href.clone()))
            .and_then(|element| {
                match href {
                    Href::Absolute(_) | Href::Relative(_) => href
                        .append_to_uri(&self.page_uri)
                        .map_err(FetchError::InvalidUriParts)
                        .and_then(|uri| {
                            self.resolver
                                .resolve_source(&uri)
                                .map_err(FetchError::ResolverError)
                                .map(|resolved| FetchResponse {
                                    body: resolved.body,
                                    mime: resolved.mime.into(),
                                })
                        }),
                    Href::Uri(uri) => ureq::get(&uri.to_string())
                        .call()
                        .map_err(|err| FetchError::Network(Box::new(err)))
                        .and_then(|resp| {
                            let mime = MediaTypeBuf::from_string(resp.content_type().to_owned())
                                .unwrap_or_else(|_| APPLICATION_OCTET_STREAM.into());

                            let mut body = vec![];

                            resp.into_reader()
                                .read_to_end(&mut body)
                                .map_err(FetchError::IoError)
                                .map(|_| FetchResponse { body, mime })
                        }),
                    Href::Other(_) => Err(FetchError::InvalidHref(href.clone())),
                }
                .map(|resp| (element, resp))
            })
            .and_then(|(element, resp)| {
                insert_response(element, resp).map_err(FetchError::Insertion)
            })
    }
}

/// Get a container element for the fetch using
/// the fetch URI file extension.
#[allow(clippy::single_call_fn)]
fn get_element_from_extension(ext: &str) -> Option<Html> {
    match ext {
        "css" | "scss" => Some(NodeRef::new_element(
            QualName::new(None, ns!(html), local_name!("style")),
            vec![],
        )),
        "html" | "svg" => Some(NodeRef::new(DocumentFragment)),
        _ => None,
    }
}

/// Insert a response into an element
#[allow(clippy::single_call_fn)]
fn insert_response(el: Html, resp: FetchResponse) -> Result<Html, InsertResponseError> {
    let contents = String::from_utf8(resp.body).unwrap_or_default();

    if resp.mime == TEXT_CSS {
        el.append(NodeRef::new_text(contents));
    } else if resp.mime == TEXT_HTML || resp.mime == IMG_SVG_XML {
        let html =
            kuchikiki::parse_fragment(QualName::new(None, ns!(html), local_name!("div")), vec![])
                .one(contents);

        let first_child = html
            .first_child()
            .ok_or(InsertResponseError(TEXT_HTML.into()))?;

        for child in first_child.children() {
            el.append(child);
        }
    } else {
        return Err(InsertResponseError(resp.mime));
    }

    Ok(el)
}

impl HtmlMod for Mod {
    fn modify(&self, html: Html) -> Result<Html, eyre::Error> {
        let nib_imports: Vec<_> = html
            .select(r#"[nib-mod~="fetch"]"#)
            .map(|sels| sels.into_iter().collect())
            .unwrap_or_default();

        for css_match in nib_imports {
            let nib_item = css_match.as_node();
            let res = nib_item
                .as_element()
                .and_then(|nib_el| nib_el.get_attr("href"))
                .and_then(|href| Href::try_from(href.as_str()).ok())
                .map(|href| self.perform_fetch(href))
                .transpose();

            if let Ok(Some(new_el)) = res {
                nib_item.insert_after(new_el);
                nib_item.detach();
            }
        }

        Ok(html)
    }
}

/// Errors possible due to a response insertion
#[derive(Debug)]
pub struct InsertResponseError(pub MediaTypeBuf);

#[allow(clippy::missing_trait_methods)]
#[allow(clippy::absolute_paths)]
impl std::error::Error for InsertResponseError {}

#[allow(clippy::absolute_paths)]
impl Display for InsertResponseError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_fmt(format_args!("Invalid mediatype: `{}`", self.0.essence()))
    }
}

/// Errors possible due to a fetch mod run
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum FetchError {
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
}

#[allow(clippy::missing_trait_methods)]
#[allow(clippy::absolute_paths)]
impl std::error::Error for FetchError {}

#[allow(clippy::absolute_paths)]
#[allow(clippy::pattern_type_mismatch)]
impl Display for FetchError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidHref(href) => formatter.write_fmt(format_args!("Invalid href: {href}")),
            Self::Insertion(ire) => ire.fmt(formatter),
            Self::InvalidUriParts(iup) => iup.fmt(formatter),
            Self::Network(neterr) => neterr.fmt(formatter),
            Self::ResolverError(resolver_error) => resolver_error.fmt(formatter),
            Self::IoError(ioerr) => ioerr.fmt(formatter),
        }
    }
}

/// The result of a fetch, either from reqaz or http
struct FetchResponse {
    /// The bytes of the response body
    body: Vec<u8>,

    /// The mime type
    mime: MediaTypeBuf,
}

use core::fmt::Display;
use crate::html::attr::{GetAttr, Href};
use crate::mediatype::{APPLICATION_OCTET_STREAM, IMG_SVG_XML, TEXT_CSS, TEXT_HTML};
use html5ever::{ns, local_name, namespace_url};
use html5ever::QualName;
use http::uri::InvalidUriParts;
use hyper::Uri;
use kuchikiki::NodeData::DocumentFragment;
use kuchikiki::traits::TendrilSink;
use mediatype::MediaTypeBuf;
use ureq::Response;
use super::HtmlMod;
use super::Html;


/// The Fetch reqaz HTML mod
#[allow(clippy::module_name_repetitions)]
pub struct FetchMod {
    /// The URI of the currently loading asset
    page_uri: Uri
}

impl FetchMod {
    #[allow(clippy::single_call_fn)]
    /// Create a new Fetch mod instance
    pub const fn new(page_uri: Uri) -> Self {
        Self {
            page_uri
        }
    }
}

/// Get a container element for the fetch using
/// the fetch URI file extension.
#[allow(clippy::single_call_fn)]
fn get_element_from_extension(ext: &str) -> Option<Html> {
    match ext {
        "css" | "scss" => {
            Some(Html::new_element(
                QualName::new(None, ns!(html), local_name!("style")),
                vec![]
            ))
        },
        "html" | "svg" => {
            Some(Html::new(DocumentFragment))
        },
        _ => None
    }
}

/// Insert a response into an element
#[allow(clippy::single_call_fn)]
fn insert_response(el: Html, resp: Response) -> Result<Html, InsertResponseError> {
    let mime = MediaTypeBuf::from_string(resp.content_type().to_owned())
        .unwrap_or_else(|_| APPLICATION_OCTET_STREAM.into());
    let contents = resp.into_string().unwrap_or_default();

    if mime == TEXT_CSS {
        el.append(Html::new_text(contents));
    } else if mime == TEXT_HTML || mime == IMG_SVG_XML {
        let html = kuchikiki::parse_html()
            .one(contents);

        for child in html.children() {
            el.append(child);
        }
    } else {
        return Err(InsertResponseError(mime));
    }
    
    Ok(el)
}

/// Complete a fetch request for an Href and Uri, returning
/// the HTML evaluated from such a fetch.
/// 
/// If style documents are fetched, they will be wrapped
/// in style tags.
#[allow(clippy::single_call_fn)]
fn perform_fetch(href: Href, page_uri: &Uri) -> Result<Html, FetchError> {
    href.extension()
        .and_then(|ext| get_element_from_extension(&ext))
        .ok_or_else(|| FetchError::InvalidHref(href.clone()))
        .and_then(|new_el| {
            href.append_to_uri(page_uri)
                .map_err(FetchError::InvalidUriParts)
                .map(|href_uri| (new_el, href_uri))
        }).and_then(|(new_el, href_uri)| {
            ureq::get(&href_uri.to_string())
                .call()
                .map_err(|err| FetchError::Network(Box::new(err)))
                .map(|resp| (new_el, resp))
        }).and_then(|(new_el, resp)| {
            insert_response(new_el, resp)
                .map_err(FetchError::Insertion)
        })
}

impl HtmlMod for FetchMod {
    fn modify(&self, html: Html) -> Result<Html, eyre::Error> {
        let nib_imports: Vec<_> = html.select(r#"[nib-mod~="fetch"]"#)
            .map(|sels| sels.into_iter().collect())
            .unwrap_or_default();

        for css_match in nib_imports {
            let nib_item = css_match.as_node();
            let res = nib_item.as_element()
                .and_then(|nib_el| nib_el.get_attr("href"))
                .and_then(|href| Href::try_from(href.as_str()).ok())
                .map(|href| perform_fetch(href, &self.page_uri))
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
pub struct InsertResponseError(MediaTypeBuf);

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
    Network(Box<ureq::Error>)
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
            Self::Network(neterr) => neterr.fmt(formatter)
        }
    }
}

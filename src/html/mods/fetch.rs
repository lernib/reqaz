use crate::html::attr::*;
use crate::mediatype::*;
use eyre::eyre;
use html5ever::{ns, local_name, namespace_url};
use html5ever::QualName;
use hyper::Uri;
use kuchikiki::NodeData::DocumentFragment;
use kuchikiki::traits::*;
use mediatype::MediaTypeBuf;
use ureq::Response;
use super::HtmlMod;
use super::Html;


pub struct FetchMod {
    page_uri: Uri
}

impl FetchMod {
    pub fn new(page_uri: Uri) -> Self {
        FetchMod {
            page_uri
        }
    }
}

pub fn get_element_from_extension(ext: String) -> Option<Html> {
    match ext.as_str() {
        "css" | "scss" => {
            return Some(Html::new_element(
                QualName::new(None, ns!(html), local_name!("style")),
                vec![]
            ));
        },
        "html" | "svg" => {
            return Some(Html::new(DocumentFragment));
        },
        _ => None
    }
}

pub fn insert_response(el: Html, resp: Response) -> Html {
    let mime = MediaTypeBuf::from_string(resp.content_type().to_string())
        .unwrap_or(APPLICATION_OCTET_STREAM.into());
    let contents = resp.into_string().unwrap_or("".into());

    if mime == TEXT_CSS {
        el.append(Html::new_text(contents));
    } else if mime == TEXT_HTML || mime == IMG_SVG_XML {
        let html = kuchikiki::parse_html()
            .one(contents);

        for child in html.children() {
            el.append(child);
        }
    }

    el
}

pub fn perform_fetch(href: Href, page_uri: &Uri) -> Result<Html, eyre::Error> {
    let new_el = href.extension()
        .and_then(get_element_from_extension)
        .ok_or(eyre!("[YOUR FAULT] nib-include only supports css"))?;

    let href_uri = href.append_to_uri(page_uri);
    let req = ureq::get(&href_uri.to_string());

    let resp = req.call()?;
    let new_el = insert_response(new_el, resp);

    Ok(new_el)
}

impl HtmlMod for FetchMod {
    fn modify(&self, html: Html) -> Result<Html, eyre::Error> {
        let nib_imports = html.select(r#"[nib-mod~="fetch"]"#)
            .and_then(|sels| Ok(sels.into_iter().collect()))
            .unwrap_or(vec![]);

        for css_match in nib_imports {
            let nib_item = css_match.as_node();
            let nib_el = nib_item.as_element()
                .ok_or(eyre!("[MY FAULT] CSS should only match elements"))?;

            let href = nib_el.get_attr("href")
                .ok_or(eyre!("[YOUR FAULT] nib-fetch MUST have an href"))?;
            
            let href = Href::try_from(href.as_str())?;

            let new_el = perform_fetch(href, &self.page_uri)?;

            nib_item.insert_after(new_el);
            nib_item.detach();
        }

        Ok(html)
    }
}

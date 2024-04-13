use color_eyre::Result;
use eyre::eyre;
use html5ever::{ns, local_name, namespace_url};
use html5ever::QualName;
use http::uri::PathAndQuery;
use hyper::Uri;
use kuchikiki::NodeRef;
use kuchikiki::traits::*;
use log::info;
use std::path::PathBuf;
use std::str::FromStr;


fn get_element_from_extension(ext: String) -> Option<NodeRef> {
    if ext == "css" {
        return Some(NodeRef::new_element(
            QualName::new(None, ns!(html), local_name!("style")),
            vec![]
        ));
    } else {
        return None;
    }
}

pub fn process_html(uri: &Uri, html: String) -> Result<String> {
    let html = kuchikiki::parse_html()
        .one(html);

    let binding = html.select_first("head")
        .map_err(|()| eyre!("[NONSENSE] All html pages need a head element"))?;
    let head = binding
        .as_node();

    let nib_imports = html.select("nib-import")
        .and_then(|sels| Ok(sels.into_iter().collect()))
        .unwrap_or(vec![]);

    for css_match in nib_imports {
        let nib_item = css_match.as_node();
        let href = nib_item.as_element()
            .and_then(|el| {
                el.attributes
                    .try_borrow()
                    .ok()
            }).and_then(|attrs| {
                let attr = attrs.get("href")?;
                // The compiler is being oh so funny
                // and not letting me clone this :hehe:
                Some(format!("{}", attr))
            }).ok_or(eyre!("[NONSENSE] nib-import MUST have href"))?;
        
        let href = Href::try_from(href.as_str())?;
        let new_el = href.extension()
            .and_then(get_element_from_extension)
            .ok_or(eyre!("[ASSUMED] nib-import only supports css"))?;

        let href_uri = href.append_to_uri(&uri);
        let contents = ureq::get(&href_uri.to_string())
            .call()?
            .into_string()?;

        new_el.append(NodeRef::new_text(contents));
        head.append(new_el);
        nib_item.detach();
    }
    
    Ok(html.to_string())
}

enum Href {
    Absolute(Uri),
    Relative(PathBuf),
    Id(String),
    Script(String)
}

impl Href {
    pub fn extension(&self) -> Option<String> {
        match self {
            Href::Absolute(uri) => PathBuf::from(uri.path())
                .extension()
                .map(|ext| ext.to_string_lossy().to_string()),
            Href::Relative(p) => p.extension()
                .map(|ext| ext.to_string_lossy().to_string()),
            _ => None
        }
    }

    pub fn append_to_uri(self, uri: &Uri) -> Uri {
        match self {
            Href::Absolute(uri) => return uri,
            Href::Relative(path) => {
                let mut parts = uri.clone().into_parts();
                let uri_path = PathBuf::from(uri.path())
                    .join(path)
                    .to_string_lossy()
                    .to_string();

                let uri_query = uri.query();
                let uri_path_query = uri_query
                    .map(|q| format!("{}?{}", uri_path, q))
                    .unwrap_or(uri_path);

                parts.path_and_query = PathAndQuery::from_str(&uri_path_query).ok();

                return Uri::from_parts(parts).expect("Should never fail to turn parts into uri");
            },
            _ => unimplemented!()
        }
    }
}

impl TryFrom<&str> for Href {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Check for starting slash
        if value.chars().next() == Some('/') {
            return Ok(Uri::from_str(value)
                .map(|uri| Href::Absolute(uri))?)
        }

        // Check for "://"
        let mut char_iter = value.chars();
        while let Some(c) = char_iter.next() {
            match c {
                'a'..='z' => continue,
                ':' => {},
                _ => break
            }

            let n1 = char_iter.next();
            let n2 = char_iter.next();

            if n1 == Some('/') && n2 == Some('/') {
                return Ok(Uri::from_str(value)
                    .map(|uri| Href::Absolute(uri))?);
            }
        }

        // Relative path (others not converted valid yet)
        let path = PathBuf::from(value);

        Ok(Href::Relative(path))
    }
}

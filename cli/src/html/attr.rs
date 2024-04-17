use http::uri::PathAndQuery;
use hyper::Uri;
use kuchikiki::ElementData;
use std::path::PathBuf;
use std::str::FromStr;


pub trait GetAttr {
    fn get_attr(&self, name: &str) -> Option<String>;
}

impl GetAttr for ElementData {
    fn get_attr(&self, name: &str) -> Option<String> {
        self.attributes
            .try_borrow()
            .ok()
            .and_then(|attrs| {
                let attr = attrs.get(name)?;
                // The compiler is being oh so funny
                // and not letting me clone this :hehe:
                Some(format!("{}", attr))
            })
    }
}

#[derive(Clone)]
pub enum Href {
    Uri(Uri),
    Absolute(PathBuf),
    Relative(PathBuf),
    Other(String)
}

impl Href {
    pub fn extension(&self) -> Option<String> {
        match self {
            Href::Uri(uri) => PathBuf::from(uri.path())
                .extension()
                .map(|ext| ext.to_string_lossy().to_string()),
            Href::Absolute(p) |
            Href::Relative(p) => p.extension()
                .map(|ext| ext.to_string_lossy().to_string()),
            _ => None
        }
    }

    pub fn append_to_uri(self, uri: &Uri) -> Uri {
        match self {
            Href::Uri(uri) => return uri,
            Href::Absolute(path) |
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
            let path = PathBuf::from(value);
            return Ok(Href::Absolute(path))
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
                    .map(|uri| Href::Uri(uri))?);
            }
        }

        // Relative path (others not converted valid yet)
        let path = PathBuf::from(value);

        Ok(Href::Relative(path))
    }
}

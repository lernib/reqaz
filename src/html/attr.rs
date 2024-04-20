use core::fmt::Display;
use core::str::FromStr;
use http::uri::{InvalidUri, InvalidUriParts, PathAndQuery};
use hyper::Uri;
use kuchikiki::ElementData;
use std::path::PathBuf;


/// Get an attribute from an element
/// 
/// This is already implemented for `kuchikiki`.
#[allow(clippy::module_name_repetitions)]
pub trait GetAttr {
    /// Get the attribute name from an element.
    fn get_attr(&self, name: &str) -> Option<String>;
}

impl GetAttr for ElementData {
    fn get_attr(&self, name: &str) -> Option<String> {
        self.attributes
            .try_borrow()
            .ok()
            .and_then(|attrs| {
                attrs.get(name).map(ToString::to_string)
            })
    }
}

/// An href URL.
/// 
/// This is used internally to tell where a resource
/// should be fetched from.
#[derive(Debug, Clone)]
pub enum Href {
    /// A full URI (https://google.com)
    Uri(Uri),

    /// An absolute path (/logo.png)
    Absolute(PathBuf),

    /// A relative path (./logo.png)
    Relative(PathBuf),

    /// Any other href entry. This can include:
    /// 
    /// - JavaScript (javascript:*)
    Other(String)
}

#[allow(clippy::absolute_paths)]
#[allow(clippy::pattern_type_mismatch)]
impl Display for Href {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Uri(uri) => uri.fmt(formatter),
            Self::Absolute(path) | Self::Relative(path) => path.to_string_lossy().fmt(formatter),
            Self::Other(str) => str.fmt(formatter)
        }
    }
}

impl Href {
    /// Get the file extension for an href url, if applicable.
    #[allow(clippy::pattern_type_mismatch)]
    pub fn extension(&self) -> Option<String> {
        match self {
            Self::Uri(uri) => PathBuf::from(uri.path())
                .extension()
                .map(|ext| ext.to_string_lossy().to_string()),
            Self::Absolute(path) |
            Self::Relative(path) => path.extension()
                .map(|ext| ext.to_string_lossy().to_string()),
            Self::Other(_) => None
        }
    }

    /// Append the href to the end of a URI, effectively
    /// resolving it relative to a location.
    pub fn append_to_uri(self, uri: &Uri) -> Result<Uri, InvalidUriParts> {
        match self {
            Self::Uri(uri_entry) => Ok(uri_entry),
            Self::Absolute(path) |
            Self::Relative(path) => {
                let mut parts = uri.clone().into_parts();
                let uri_path = PathBuf::from(uri.path())
                    .join(path)
                    .to_string_lossy()
                    .to_string();

                let uri_query = uri.query();
                let uri_path_query = uri_query
                    .map(|query| format!("{uri_path}?{query}"))
                    .unwrap_or(uri_path);

                parts.path_and_query = PathAndQuery::from_str(&uri_path_query).ok();

                Uri::from_parts(parts)
            },
            Self::Other(_) => Ok(uri.clone())
        }
    }
}

impl TryFrom<&str> for Href {
    type Error = InvalidUri;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Check for javascript:
        if value.starts_with("javascript:") {
            return Ok(Self::Other(value.to_owned()))
        }
        
        // Check for starting slash
        if value.starts_with('/') {
            let path = PathBuf::from(value);
            return Ok(Self::Absolute(path))
        }

        // Check for "://"
        if value.contains("://") {
            return Uri::from_str(value).map(Self::Uri);
        }

        // Relative path (others not converted valid yet)
        let path = PathBuf::from(value);

        Ok(Self::Relative(path))
    }
}

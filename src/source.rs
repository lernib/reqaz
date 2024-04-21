extern crate alloc;

use alloc::sync::Arc;
use core::fmt::Display;
use core::future::Future;
use core::pin::Pin;
use crate::html::process_html;
use crate::mediatype::{GetMediaType, TEXT_HTML};
use color_eyre::owo_colors::OwoColorize;
use http::uri::{Authority, InvalidUriParts, Scheme};
use http_body_util::Full;
use hyper::{Request, Response, StatusCode, Uri};
use hyper::body::Bytes;
use hyper::body::Incoming as IncomingBody;
use hyper::service::Service;
use mediatype::MediaType;
use rsass::output::Format as RsassFormat;
use std::io::ErrorKind as IoErrorKind;
use std::path::{Path, PathBuf};


/// The source service, used with hyper
#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct SourceService {
    /// The resolver to use. This is made by the client of
    /// the library, and is supplied at creation time.
    resolver: Arc<SourceResolver>,

    /// Whether to log or not
    log: bool
}



impl SourceService {
    /// Create a new source service
    #[inline]
    pub fn new(resolver: SourceResolver, log: bool) -> Self {
        Self {
            resolver: Arc::new(resolver),
            log
        }
    }

    /// Handle a hyper request, passed by the service trait
    #[allow(clippy::unused_async)]
    async fn handle_request(&self, req: Request<IncomingBody>) ->
        // type safety 😌
        Result<<&Self as Service<Request<IncomingBody>>>::Response, <&Self as Service<Request<IncomingBody>>>::Error>
    {
        let req_path = req
            .uri()
            .path_and_query()
            .map(ToString::to_string);

        let source = self.resolver.resolve_source(req.uri());

        let response = Response::builder();
        
        match source {
            Err(err) => {
                match err {
                    ResolverError::NotFound => self.log_source_request(StatusCode::NOT_FOUND, req_path),
                    ResolverError::ServerIssue |
                    ResolverError::InvalidUriParts(_) |
                    ResolverError::NoMimeFound |
                    ResolverError::WasNotUtf8 |
                    ResolverError::ModProblem(_) |
                    ResolverError::ParseAsMime |
                    ResolverError::Http(_) => self.log_source_request(StatusCode::INTERNAL_SERVER_ERROR, req_path)
                }

                Err(err)
            },
            Ok(Resolved {
                body,
                mime
            }) => {
                if self.log {
                    #[allow(clippy::print_stdout)]
                    if let Some(path) = req_path {
                        println!(
                            "[{}] {}",
                            200i16.green().bold(),
                            path
                        );
                    }
                }

                response.status(200)
                    .header("Content-Type", mime.to_string())
                    .body(Full::new(Bytes::from(body)))
                    .map_err(ResolverError::Http)
            }
        }
    }

    /// Log a request to the console, if logging is enabled
    fn log_source_request(&self, status: StatusCode, req_path: Option<String>) {
        if self.log {
            let status_colored = match status.as_u16() {
                100..=199 => status.blue().to_string(),
                300..=399 => status.yellow().to_string(),
                400..=499 => status.red().to_string(),
                500..=599 => status.purple().to_string(),
                _ => status.to_string()
            };
        
            #[allow(clippy::print_stdout)]
            if let Some(path) = req_path {
                println!(
                    "[{}] {}",
                    status_colored.bold(),
                    path
                );
            }
        }
    }
}

impl<'me> Service<Request<IncomingBody>> for &'me SourceService {
    type Response = Response<Full<Bytes>>;
    type Error = ResolverError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'me>>;

    #[inline]
    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        Box::pin(self.handle_request(req))
    }
}

/// Resolver for `SourceService`
#[derive(Clone)]
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
pub struct SourceResolver {
    /// Root to serve from
    pub root: PathBuf,

    /// URL being served from, used to base path fetches
    pub authority: Authority
}

impl SourceResolver {
    /// Create a source resolver
    #[inline]
    pub const fn new(root: PathBuf, authority: Authority) -> Self {
        Self {
            root,
            authority
        }
    }

    /// Resolve source content from request object from URI
    /// 
    /// # Errors
    /// 
    /// Any errors that occur while resolving the URI are propogated
    #[inline]
    pub fn resolve_source(&self, uri: &Uri) -> Result<Resolved, ResolverError> {
        let uri_old: Uri = uri.clone();

        let mut parts = uri_old.into_parts();
        parts.scheme = Some(Scheme::HTTP);
        parts.authority = Some(self.authority.clone());

        let path = self.get_path_from_uri(uri);

        #[allow(clippy::absolute_paths)]
        let src_mime_uri_fallible = std::fs::read(&path)
            .map_err(|err| {
                #[allow(clippy::wildcard_enum_match_arm)]
                match err.kind() {
                    IoErrorKind::NotFound => ResolverError::NotFound,
                    _ => ResolverError::ServerIssue
                }
            }).and_then(|src |{
                path.get_media_type()
                    .clone()
                    .ok_or(ResolverError::NoMimeFound)
                    .map(|mime| (src, mime))
            }).and_then(|(src, mime)| {
                Uri::from_parts(parts)
                    .map_err(ResolverError::InvalidUriParts)
                    .map(|uri_new| (src, mime, uri_new))
            });
        
        match src_mime_uri_fallible {
            Ok((src, mime, uri_new)) => {
                let body = {
                    if mime == TEXT_HTML {
                        let body_str_fallible = String::from_utf8(src)
                            .map_err(|_err| ResolverError::WasNotUtf8);

                        match body_str_fallible {
                            Ok(body_str) => {
                                process_html(self, &uri_new, body_str)
                                    .map_err(ResolverError::ModProblem)
                            },
                            Err(err) => Err(err)
                        }.map(|new_body| new_body.bytes().collect())
                    } else if Path::new(uri.path())
                        .extension()
                        .map_or(false, |ext| ext.eq_ignore_ascii_case("scss")) {
                        rsass::compile_scss(&src, RsassFormat::default())
                            .map_err(|_err| ResolverError::ParseAsMime)
                    } else {
                        Ok(src)
                    }
                };

                body.map(|body_vec| Resolved {
                    body: body_vec,
                    mime
                })
            },
            Err(err) => Err(err)
        }
    }

    /// Get the path for a resource request
    fn get_path_from_uri(&self, uri: &Uri) -> PathBuf {
        let pathname = uri.path()
            .get(1..)
            .unwrap_or("")
            .to_owned();

        let path = self.root.join(pathname);
        
        let pages_path = path.join("index.html");

        #[allow(clippy::absolute_paths)]
        if pages_path.exists() {
            pages_path
        } else {
            path
        }
    }
}

/// A resolved resource
#[non_exhaustive]
pub struct Resolved {
    /// The body of the resolved resource, as bytes
    pub body: Vec<u8>,

    /// The mime type
    pub mime: MediaType<'static>
}

/// Any error that can be returned by the source resolver
#[derive(Debug)]
#[non_exhaustive]
pub enum ResolverError {
    /// Could not properly construct a URI
    InvalidUriParts(InvalidUriParts),

    /// Mime could not be found (did you try to resolve a folder without an index.html?)
    NoMimeFound,

    /// Resource not found
    NotFound,

    /// There was a server issue
    ServerIssue,

    /// Expected UTF8, but resource contents were not
    WasNotUtf8,

    /// There was an error running a mod
    ModProblem(eyre::Report),

    /// There was a problem parsing an expected mime type
    ParseAsMime,

    /// HTTP problems
    Http(http::Error)
}

#[allow(clippy::missing_trait_methods)]
#[allow(clippy::absolute_paths)]
impl std::error::Error for ResolverError {}

#[allow(clippy::absolute_paths)]
#[allow(clippy::pattern_type_mismatch)]
impl Display for ResolverError {
    #[inline]
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidUriParts(iup) => iup.fmt(formatter),
            Self::NoMimeFound => formatter.write_str("No mime found (did you try to resolve a folder without an index.html?)"),
            Self::NotFound => formatter.write_str("Resource not found"),
            Self::ServerIssue => formatter.write_str("There was a server issue"),
            Self::WasNotUtf8 => formatter.write_str("Expected UTF8, but resource contents were not"),
            Self::ModProblem(err) => formatter.write_fmt(format_args!("There was a mod problem: {err}")),
            Self::ParseAsMime => formatter.write_str("There was a problem parsing an expected mime type"),
            Self::Http(err) => err.fmt(formatter)
        }
    }
}

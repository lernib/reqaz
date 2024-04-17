use crate::html::process_html;
use crate::mediatype::GetMediaType;
use http::uri::{Authority, Scheme};
use http_body_util::Full;
use hyper::{Request, Response, StatusCode, Uri};
use hyper::body::Bytes;
use hyper::body::Incoming as IncomingBody;
use hyper::service::Service;
use mediatype::{media_type, MediaType};
use std::future::Future;
use std::io::ErrorKind as IoErrorKind;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;


#[derive(Clone)]
pub struct SourceService {
    resolver: Arc<SourceResolver>
}

impl SourceService {
    pub fn new(src: PathBuf, authority: Authority) -> Self {
        let resolver = SourceResolver {
            src,
            authority
        };

        Self {
            resolver: Arc::new(resolver)
        }
    }

    async fn handle_request(&self, req: Request<IncomingBody>) ->
        // type safety 😌
        Result<<&Self as Service<Request<IncomingBody>>>::Response, <&Self as Service<Request<IncomingBody>>>::Error>
    {
        let source = self.resolver.resolve_source(req).await;

        let response = Response::builder();
        let response = match source {
            ResolvedSource::Fail {
                status
            } => {
                response.status(status)
                    .body(Full::new(Bytes::default()))
            },
            ResolvedSource::Success {
                body,
                mime
            } => {
                response.status(200)
                    .header("Content-Type", mime.to_string())
                    .body(Full::new(Bytes::from(body)))
            }
        }?;

        Ok(response)
    }
}

impl<'me> Service<Request<IncomingBody>> for &'me SourceService {
    type Response = Response<Full<Bytes>>;
    type Error = color_eyre::Report;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'me>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        Box::pin(self.handle_request(req))
    }
}

#[derive(Clone)]
struct SourceResolver {
    pub src: PathBuf,
    pub authority: Authority
}

impl SourceResolver {
    async fn resolve_source(&self, req: Request<IncomingBody>) -> ResolvedSource {
        let uri = req.uri();

        let mut parts = uri.clone().into_parts();
        parts.scheme = Some(Scheme::HTTP);
        parts.authority = Some(self.authority.clone());

        let uri = Uri::from_parts(parts)
            .unwrap_or_else(|_| unreachable!());

        let path = self.get_path_from_request(req).await;
        let mime = path.get_media_type().clone();

        if let Some(mime) = mime {
            return tokio::fs::read(path)
                .await
                .map_or_else(
                    |e| {
                        let status = match e.kind() {
                            IoErrorKind::NotFound => StatusCode::NOT_FOUND,
                            _ => StatusCode::INTERNAL_SERVER_ERROR
                        };

                        ResolvedSource::Fail {
                            status
                        }
                    },
                    |s| {
                        let mut body = s;

                        if mime == media_type!(TEXT/HTML) {
                            let body_str = String::from_utf8(body);
                            if let Ok(body_str) = body_str {

                                let new_body = process_html(&uri, body_str);

                                if let Ok(b) = new_body {
                                    body = b.bytes().collect()
                                } else {
                                    eprintln!("Error serving request: {}", new_body.unwrap_err());

                                    return ResolvedSource::Fail {
                                        status: StatusCode::FAILED_DEPENDENCY
                                    }
                                }
                            } else {
                                return ResolvedSource::Fail {
                                    status: StatusCode::EXPECTATION_FAILED
                                }
                            }
                        } else if uri.path().ends_with(".scss") {
                            let compiled = rsass::compile_scss(&body, Default::default());

                            match compiled {
                                Ok(compiled) => body = compiled,
                                Err(e) => {
                                    log::error!("Sass problem: {}", e);
                                    return ResolvedSource::Fail {
                                        status: StatusCode::EXPECTATION_FAILED
                                    }
                                }
                            }
                        }

                        ResolvedSource::Success {
                            body,
                            mime: mime.into()
                        }
                    }
                )
        }

        ResolvedSource::Fail { status: StatusCode::INTERNAL_SERVER_ERROR }
    }

    async fn get_path_from_request(&self, req: Request<IncomingBody>) -> PathBuf {
        let uri = req.uri();
        let mut base_folder = "static";

        if req.headers()
            .get("Nib-Variant")
            .and_then(|hv| hv.to_str().ok()) == Some("component")
        {
            base_folder = "components";
        }

        let pathname = uri.path()
            .get(1..)
            .unwrap_or("")
            .to_owned();

        let pages_path = self.src
            .join("pages")
            .join(&pathname)
            .join("index.html");

        if tokio::fs::try_exists(&pages_path).await.unwrap_or(false) {
            return pages_path;
        } else {
            return self.src
                .join(base_folder)
                .join(pathname);
        }
    }
}

enum ResolvedSource {
    Fail {
        status: StatusCode
    },

    Success {
        body: Vec<u8>,
        mime: MediaType<'static>
    }
}

use mime::Mime;
use std::path::Path;


pub trait GetMime {
    fn get_mime(&self) -> Option<Mime>;
}

impl GetMime for Path {
    fn get_mime(&self) -> Option<Mime> {
        let ext = self.extension()?.to_str()?;

        Some(match ext {
            "html" => mime::TEXT_HTML,
            "css" => mime::TEXT_CSS,
            _ => mime::APPLICATION_OCTET_STREAM
        })
    }
}

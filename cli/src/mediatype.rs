use mediatype::MediaType;
use mediatype::media_type;
use std::path::Path;

pub const TEXT_HTML: MediaType<'_> = media_type!(TEXT/HTML);
pub const TEXT_CSS: MediaType<'_> = media_type!(TEXT/CSS);
pub const APPLICATION_OCTET_STREAM: MediaType<'_> = media_type!(APPLICATION/OCTET_STREAM);


pub trait GetMediaType {
    fn get_media_type(&self) -> Option<MediaType<'static>>;
}

impl GetMediaType for Path {
    fn get_media_type(&self) -> Option<MediaType<'static>> {
        let ext = self.extension()?.to_str()?;

        Some(match ext {
            "html" => TEXT_HTML,
            "css" => TEXT_CSS,
            _ => APPLICATION_OCTET_STREAM
        })
    }
}

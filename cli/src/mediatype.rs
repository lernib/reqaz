use mediatype::MediaType;
use mediatype::media_type;
use std::path::Path;

pub const TEXT_HTML: MediaType<'_> = media_type!(TEXT/HTML);
pub const TEXT_CSS: MediaType<'_> = media_type!(TEXT/CSS);
pub const IMG_SVG_XML: MediaType<'_> = media_type!(IMAGE/SVG+XML);
pub const IMG_JPEG: MediaType<'_> = media_type!(IMAGE/JPEG);
pub const IMG_PNG: MediaType<'_> = media_type!(IMAGE/PNG);
pub const IMG_WEBP: MediaType<'_> = media_type!(IMAGE/WEBP);
pub const IMG_GIF: MediaType<'_> = media_type!(IMAGE/GIF);
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
            "scss" => TEXT_CSS,
            "svg" => IMG_SVG_XML,
            "jpeg" => IMG_JPEG,
            "png" => IMG_PNG,
            "webp" => IMG_WEBP,
            "gif" => IMG_GIF,
            _ => APPLICATION_OCTET_STREAM
        })
    }
}

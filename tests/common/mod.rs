use kuchikiki::traits::TendrilSink;
use std::path::PathBuf;

pub fn serve_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("e2e/web")
}

pub fn expected_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("e2e/expected")
}

pub fn parse_html_string(s: &str) -> reqaz::html::Html {
    kuchikiki::parse_html().one(s)
}

pub fn without_newlines(s: &str) -> String {
    s.replace('\n', "").to_string()
}

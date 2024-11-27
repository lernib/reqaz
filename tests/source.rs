use reqaz::source::SourceResolver;
use std::fs::File;
use std::io::Read;

mod common;

#[test]
fn source_resolver_basics() {
    let serve_dir = common::serve_dir();

    let resolver = SourceResolver::new(serve_dir.clone(), "reqaz.local".try_into().unwrap());

    let out = resolver
        .resolve_source(&"/basic.html".try_into().unwrap())
        .unwrap();

    let mut basic_html = File::open(serve_dir.join("basic.html")).unwrap();

    let mut basic_html_content = String::default();
    basic_html.read_to_string(&mut basic_html_content).unwrap();

    let basic_html_content = common::parse_html_string(&basic_html_content).to_string();

    assert_eq!(
        basic_html_content,
        std::str::from_utf8(&out.body).unwrap().to_string()
    )
}

#[test]
fn source_fetch() {
    let serve_dir = common::serve_dir();
    let expected_dir = common::expected_dir();

    let resolver = SourceResolver::new(serve_dir, "reqaz.local".try_into().unwrap());

    let out = resolver
        .resolve_source(&"/fetch.html".try_into().unwrap())
        .unwrap();

    let mut expected_file = File::open(expected_dir.join("fetch.html.diff")).unwrap();

    let mut html_content = String::default();
    expected_file.read_to_string(&mut html_content).unwrap();

    let html_content = common::parse_html_string(&html_content).to_string();

    assert_eq!(
        common::without_newlines(&html_content),
        common::without_newlines(std::str::from_utf8(&out.body).unwrap())
    )
}

#[test]
fn source_fetch_css() {
    let serve_dir = common::serve_dir();
    let expected_dir = common::expected_dir();

    let resolver = SourceResolver::new(serve_dir, "reqaz.local".try_into().unwrap());

    let out = resolver
        .resolve_source(&"/fetch_css.html".try_into().unwrap())
        .unwrap();

    let mut expected_file = File::open(expected_dir.join("fetch_css.html.diff")).unwrap();

    let mut html_content = String::default();
    expected_file.read_to_string(&mut html_content).unwrap();

    let html_content = common::parse_html_string(&html_content).to_string();

    assert_eq!(
        common::without_newlines(&html_content),
        common::without_newlines(std::str::from_utf8(&out.body).unwrap())
    )
}

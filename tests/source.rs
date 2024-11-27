use reqaz::source::SourceResolver;
use std::fs::File;
use std::io::Read;

mod common;

macro_rules! source_test_list {
    ($([$name:ident, $path:literal]),+) => {
        $(
            ::paste::paste! {
                #[test]
                fn [< source_ $name >]() {
                    let serve_dir = common::serve_dir();
                    let expected_dir = common::expected_dir();

                    let resolver = SourceResolver::new(serve_dir, "reqaz.local".try_into().unwrap());

                    let out = resolver
                        .resolve_source(&concat!("/", $path).try_into().unwrap())
                        .unwrap();

                    let mut expected_file = File::open(expected_dir.join(concat!($path, ".diff"))).unwrap();

                    let mut html_content = String::default();
                    expected_file.read_to_string(&mut html_content).unwrap();

                    let html_content = common::parse_html_string(&html_content).to_string();

                    assert_eq!(
                        common::without_newlines(&html_content),
                        common::without_newlines(std::str::from_utf8(&out.body).unwrap())
                    )
                }
            }
        )+
    };
}

source_test_list![
    [basic, "basic.html"],
    [fetch, "fetch.html"],
    [fetch_css, "fetch_css.html"],
    [component_basic, "component/basic_component.html"],
    [
        component_with_other_content,
        "component/with_other_content.html"
    ]
];

use super::Html;
use super::HtmlMod;
use core::fmt::Display;
use eyre::Result;
use html5ever::QualName;
use html5ever::{local_name, namespace_url, ns};
use lightningcss::error::Error as CssError;
use lightningcss::error::MinifyErrorKind;
use lightningcss::error::ParserError;
use lightningcss::error::PrinterErrorKind;
use lightningcss::printer::PrinterOptions;
use lightningcss::stylesheet::MinifyOptions;
use lightningcss::stylesheet::ParserOptions;
use lightningcss::stylesheet::StyleSheet;

/// The CSS reqaz HTML mod
///
/// This mod takes no arguments and is constructed
/// using Default, but it needs a struct to implement
/// `HtmlMod`.
#[derive(Default)]
pub struct Mod;

/// Minify a CSS string
#[allow(clippy::single_call_fn)]
fn minify_css(css: &str) -> Result<String, MinifyCssError> {
    let printer_options = PrinterOptions {
        minify: true,
        ..Default::default()
    };

    StyleSheet::parse(css, ParserOptions::default())
        .map_err(|err| MinifyCssError::Parse(err.into_owned()))
        .and_then(|mut stylesheet| {
            stylesheet
                .minify(MinifyOptions::default())
                .map_err(MinifyCssError::Minify)
                .map(|()| stylesheet)
        })
        .and_then(|stylesheet| {
            stylesheet
                .to_css(printer_options)
                .map_err(MinifyCssError::Print)
        })
        .map(|res| res.code)
}

impl HtmlMod for Mod {
    fn modify(&self, html: super::Html) -> Result<Html, eyre::Error> {
        let styles: Vec<_> = html
            .select("style")
            .map(|sels| sels.into_iter().collect())
            .unwrap_or_default();

        let mut combined = String::new();

        for css_match in styles {
            combined += css_match.text_contents().as_str();
            css_match.as_node().detach();
        }

        minify_css(&combined)
            .map(|css| {
                let binding = html.select_first("head").ok();
                let head = binding.map(|node_data| node_data.as_node().to_owned());

                let style_node =
                    Html::new_element(QualName::new(None, ns!(html), local_name!("style")), vec![]);

                style_node.append(Html::new_text(css));

                if let Some(head_ref) = head {
                    head_ref.append(style_node);
                } else {
                    html.append(style_node);
                }

                html
            })
            .map_err(Into::into)
    }
}

/// Possible errors from running the `minify_css` function
#[derive(Debug)]
enum MinifyCssError {
    /// Errors while parsing CSS string
    Parse(CssError<ParserError<'static>>),

    /// Errors while minifying CSS
    Minify(CssError<MinifyErrorKind>),

    /// Errors while printing new CSS string
    Print(CssError<PrinterErrorKind>),
}

#[allow(clippy::missing_trait_methods)]
#[allow(clippy::absolute_paths)]
impl std::error::Error for MinifyCssError {}

#[allow(clippy::absolute_paths)]
#[allow(clippy::pattern_type_mismatch)]
impl Display for MinifyCssError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(err) => err.fmt(formatter),
            Self::Minify(err) => err.fmt(formatter),
            Self::Print(err) => err.fmt(formatter),
        }
    }
}

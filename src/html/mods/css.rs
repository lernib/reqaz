use eyre::Result;
use html5ever::{ns, local_name, namespace_url};
use html5ever::QualName;
use lightningcss::printer::PrinterOptions;
use lightningcss::stylesheet::MinifyOptions;
use lightningcss::stylesheet::ParserOptions;
use lightningcss::stylesheet::StyleSheet;
use super::HtmlMod;
use super::Html;


#[derive(Default)]
pub struct CssMod;

fn minify_css(css: &str) -> Result<String> {
    let mut stylesheet = StyleSheet::parse(
        &css,
        ParserOptions::default()
    ).map_err(|e| e.into_owned())?;

    let options = MinifyOptions::default();
    stylesheet.minify(options)?;

    let options = PrinterOptions {
        minify: true,
        ..Default::default()
    };

    Ok(stylesheet.to_css(options)?.code)
}

impl HtmlMod for CssMod {
    fn modify(&self, html: super::Html) -> Result<Html> {
        let styles = html.select(r#"style"#)
            .and_then(|sels| Ok(sels.into_iter().collect()))
            .unwrap_or(vec![]);

        let mut combined = String::new();

        for css_match in styles {
            combined += css_match.text_contents().as_str();
            css_match.as_node().detach();
        }

        let css = minify_css(&combined)?;

        let binding = html.select_first("head")
            .unwrap_or_else(|_| unreachable!());
        let head = binding
            .as_node();

        let style_node = Html::new_element(
            QualName::new(None, ns!(html), local_name!("style")),
            vec![]
        );

        style_node.append(Html::new_text(css));

        head.append(style_node);

        Ok(html)
    }
}

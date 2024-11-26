use self::mods::HtmlModManager;
use crate::source::SourceResolver;
use hyper::Uri;
use kuchikiki::traits::TendrilSink;

/// Utilities for HTML element attributes
mod attr;

/// reqaz-builtin HTML mods
pub(crate) mod mods;

pub type Html = kuchikiki::NodeRef;

/// Apply internal mods to a parsed HTML segment
macro_rules! apply_internal_mods {
    ($resolver:ident, $uri:ident, $dom:ident, [$($mod_name:literal),*]) => {
        {
            let mut mod_manager = HtmlModManager {
                page_uri: $uri.clone(),
                mod_cache: Default::default(),
                resolver: $resolver.clone()
            };

            mod_manager.load_mods([$($mod_name),*]);

            Ok($dom)
                $(
                    .and_then(|new_dom| mod_manager.apply_mod(new_dom, $mod_name))
                )*
                .map(|new_dom| new_dom.to_string())
        }
    };
}

/// Process HTML using reqaz-builtin mods and kuchikiki
///
/// # Errors
///
/// Any mod errors are propagated up to the caller.
#[inline]
#[allow(clippy::module_name_repetitions)]
pub fn process_html(
    resolver: &SourceResolver,
    uri: &Uri,
    html: String,
) -> Result<String, mods::Error> {
    let dom = kuchikiki::parse_html().one(html);

    apply_internal_mods!(resolver, uri, dom, ["fetch", "css", "component"])
}

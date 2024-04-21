use eyre::eyre;
use hyper::Uri;
use std::collections::HashMap;
use crate::source::SourceResolver;

use super::Html;

/// The CSS internal mod, which bundles all
/// `style` tags into one and minifies them.
/// This should be applied after all fetches.
mod css;

/// The fetch internal mod, which fetches any
/// resources with an `href` and fetch mod request.
mod fetch;

/// reqaz HTML mod Error type
pub type Error = eyre::Report;

/// The reqaz HTML mod trait
pub trait HtmlMod {
    /// Modify some HTML, and return the result.
    fn modify(&self, html: Html) -> Result<Html, Error>;
}

/// An HTML mod manager, used to load mods ahead of time without
/// creating them multiple times per request.
pub struct HtmlModManager {
    /// The URI of the currently loading asset
    pub page_uri: Uri,

    /// The mod cache
    pub mod_cache: HashMap<String, Box<dyn HtmlMod>>,

    /// A resolver, for the fetch internal mod
    pub resolver: SourceResolver
}

impl HtmlModManager {
    /// Load an internal mod
    fn load_mod(&mut self, mod_name: &str) -> Option<Box<dyn HtmlMod>> {
        let mod_box: Box<dyn HtmlMod> = match mod_name {
            "fetch" => Box::new(fetch::Mod::new(self.page_uri.clone(), self.resolver.clone())),
            "css" => Box::<css::Mod>::default(),
            _ => return None
        };

        Some(mod_box)
    }

    /// Load a set of internal mods
    pub fn load_mods<const N: usize>(&mut self, mods: [&str; N]) {
        for mod_name in mods {
            if let Some(mod_) = self.load_mod(mod_name) {
                self.mod_cache.insert(
                    mod_name.to_owned(),
                    mod_
                );
            }
        }
    }

    /// Get a specific internal mod
    fn get_mod(&self, mod_name: &str) -> Option<&Box<dyn HtmlMod>> {
        self.mod_cache.get(mod_name)
    }

    /// Apply a mod to an HTML fragment, returning the result
    pub fn apply_mod(&self, html: Html, mod_name: &str) -> Result<Html, Error> {
        self.get_mod(mod_name)
            .ok_or(eyre!("Mod does not exist"))
            .and_then(|mod_| mod_.modify(html))
    }
}

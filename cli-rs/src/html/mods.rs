use eyre::eyre;
use eyre::Result;
use hyper::Uri;
use std::collections::HashMap;

pub use super::Html;

mod css;
mod fetch;
mod script;


pub trait HtmlMod {
    fn modify(&self, html: Html) -> Result<Html>;
}

pub struct HtmlModManager {
    page_uri: Uri,
    mod_cache: HashMap<String, Box<dyn HtmlMod>>
}

impl HtmlModManager {
    pub fn new(page_uri: Uri) -> Self {
        HtmlModManager {
            page_uri,
            mod_cache: Default::default()
        }
    }

    fn load_mod(&mut self, mod_: &str) -> Option<Box<dyn HtmlMod>> {
        let mod_: Box<dyn HtmlMod> = match mod_ {
            "fetch" => Box::new(fetch::FetchMod::new(self.page_uri.clone())),
            "css" => Box::new(css::CssMod::default()),
            "script" => Box::new(script::ScriptMod::new(self.page_uri.clone())),
            _ => return None
        };

        Some(mod_)
    }

    pub fn load_mods<const N: usize>(&mut self, mods: [&str; N]) {
        for mod_name in mods {
            if let Some(mod_) = self.load_mod(mod_name) {
                self.mod_cache.insert(
                    mod_name.to_string(),
                    mod_
                );
            }
        }
    }

    fn get_mod(&self, mod_name: &str) -> Option<&Box<dyn HtmlMod>> {
        self.mod_cache.get(mod_name)
    }

    pub fn apply_mod(&self, html: Html, mod_name: &str) -> Result<Html> {
        self.get_mod(mod_name)
            .ok_or(eyre!("Mod does not exist"))
            .map(|mod_| mod_.modify(html))?
    }
}

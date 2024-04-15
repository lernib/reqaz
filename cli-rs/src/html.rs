use color_eyre::Result;
use hyper::Uri;
use kuchikiki::traits::*;
use self::mods::HtmlModManager;

mod attr;
mod mods;


pub type Html = kuchikiki::NodeRef;

const MOD_NAMES: [&str; 2] = ["fetch", "css"];

pub fn process_html(uri: &Uri, html: String) -> Result<String> {
    let mut mod_manager = HtmlModManager::new(uri.clone());
    mod_manager.load_mods(MOD_NAMES);

    let mut html = kuchikiki::parse_html()
        .one(html);

    for mod_name in MOD_NAMES {
        html = mod_manager.apply_mod(html, mod_name)?;
    }

    Ok(html.to_string())
}

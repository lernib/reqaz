//! # reqaz
//!
//! Requests from A to Z (reqaz) is a tool to help manage varions aspects of static HTML pages. We use it to help bundle things like CSS and certain HTML assets ahead of time before deploying to a bucket.
//!
//! This isn't quite ready to use, but it's almost ready for us to use. Once it is, we will provide instructions for others as well.

#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
// Current requirement, might fix later idk
#![allow(clippy::multiple_crate_versions)]
// Remove clippy contradictions here
#![allow(clippy::blanket_clippy_restriction_lints)]
#![allow(clippy::implicit_return)]
#![allow(clippy::self_named_module_files)]
#![allow(clippy::unseparated_literal_suffix)]
#![allow(clippy::pub_with_shorthand)]

pub mod html;
pub mod mediatype;
pub mod source;

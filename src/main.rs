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
#![allow(clippy::unseparated_literal_suffix)]

use clap::Parser;
use color_eyre::Result;
use core::net::SocketAddr;
use core::str::FromStr;
use eyre::eyre;
use http::uri::Authority;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use reqaz::source::{SourceResolver, SourceService};
use std::env::current_dir;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::task::spawn as tokio_spawn;


/// CLI parsing struct, courtesy of clap
#[derive(Parser)]
struct Cli {
    /// The path to serve from
    #[arg(
        short = 'C'
    )]
    path: Option<PathBuf>,

    /// The port to serve from
    #[arg(
        short = 'p',
        long = "port",
        default_value = "5000"
    )]
    port: u16,

    /// Whether to print logs on request status
    #[arg(
        long = "log"
    )]
    log: bool
}

#[tokio::main]
#[allow(clippy::question_mark_used)]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let authority = Authority::from_str(
        &format!("localhost:{}", args.port)
    )?;

    color_eyre::install()?;

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = TcpListener::bind(addr).await?;

    let root = args.path.or_else(|| {
        current_dir().ok()
    }).ok_or(eyre!("No root path provided"))?;

    let service = SourceService::new(
        SourceResolver::new(root, authority),
        args.log
    );

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let service_clone = service.clone();

        #[allow(clippy::print_stderr)]
        tokio_spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, &service_clone)
                .await
            {
                eprintln!("Error serving request: {err}");
            }
        });
    }
}

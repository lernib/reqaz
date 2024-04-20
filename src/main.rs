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
use serde::{Serialize, Deserialize};
use std::env::current_dir;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::task::spawn as tokio_spawn;


/// Requests from A to Z
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
    )]
    port: Option<u16>,

    /// Whether to print logs on request status
    #[arg(
        long = "log"
    )]
    log: Option<bool>
}

#[tokio::main]
#[allow(clippy::question_mark_used)]
#[allow(clippy::absolute_paths)]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();
    let config = {
        let base_config = CliConfig::default();

        if PathBuf::from("./reqaz.json").exists() {
            let json_contents = tokio::fs::read_to_string("./reqaz.json").await?;
            let json_config: CliConfig = serde_json::from_str(&json_contents)?;

            Ok::<CliConfig, eyre::Error>(json_config.override_with_cli(args))
        } else {
            Ok(base_config.override_with_cli(args))
        }
    }?;

    let authority = Authority::from_str(
        &format!("localhost:{}", config.port)
    )?;

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr).await?;

    let root = config.root.or_else(|| {
        current_dir().ok()
    }).ok_or(eyre!("No root path provided"))?;

    let service = SourceService::new(
        SourceResolver::new(root, authority),
        config.log
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

/// Base CLI configuration (reqaz.json)
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
struct CliConfig {
    /// The root folder to serve from
    pub root: Option<PathBuf>,

    /// The port to serve from
    pub port: u16,

    /// Enable logging
    pub log: bool
}

impl CliConfig {
    /// Override config with CLI options manually
    pub fn override_with_cli(mut self, cli: Cli) -> Self {
        if let Some(root) = cli.path {
            self.root = Some(root);
        }

        if let Some(port) = cli.port {
            self.port = port;
        }

        if let Some(log) = cli.log {
            self.log = log;
        }

        self
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            root: None,
            port: 5000,
            log: false
        }
    }
}

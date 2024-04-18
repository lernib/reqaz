use clap::Parser;
use color_eyre::Result;
use http::uri::Authority;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use source::{SourceResolver, SourceService};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::net::TcpListener;

mod html;
mod mediatype;
mod source;


#[derive(Parser)]
struct Cli {
    path: PathBuf,

    #[arg(
        long = "framework",
        help = "Enable framework mode [TESTING ONLY]"
    )]
    framework: bool,

    #[arg(
        short = 'p',
        long = "port",
        default_value = "5000"
    )]
    port: u16,

    #[arg(
        long = "log"
    )]
    log: bool
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let authority = Authority::from_str(
        &format!("localhost:{}", args.port)
    )?;

    color_eyre::install()?;

    if args.log {
        colog::init();
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = TcpListener::bind(addr).await?;
    let src = if args.framework {
        args.path.join("src")
    } else {
        args.path
    };

    let service = SourceService::new(
        SourceResolver {
            src,
            authority,
            framework: args.framework
        }
    );

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let service = service.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, &service)
                .await
            {
                eprintln!("Error serving request: {}", err)
            }
        });
    }
}

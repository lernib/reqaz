use clap::Parser;
use color_eyre::Result;
use http::uri::Authority;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use source::SourceService;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;

mod html;
mod mime;
mod source;


#[derive(Parser)]
struct Cli {
    path: PathBuf
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let authority = Authority::from_static("localhost:5000");

    color_eyre::install()?;
    colog::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    let listener = TcpListener::bind(addr).await?;

    let service = SourceService::new(
        args.path.join("src"),
        authority
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

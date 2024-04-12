use clap::Parser;
use color_eyre::Result;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use source::SourceService;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;

mod mime;
mod source;


#[derive(Parser)]
struct Cli {
    path: PathBuf
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    color_eyre::install()?;

    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    let listener = TcpListener::bind(addr).await?;

    let service = SourceService::new(args.path.join("src"));

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let service = service.clone();

        tokio::task::spawn(async move {
            if let Err(_err) = http1::Builder::new()
                .serve_connection(io, &service)
                .await
            {
                eprintln!("Error serving request")
            }
        });
    }
}

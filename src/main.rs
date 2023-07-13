mod error;
mod server;

pub mod daemon_handle;
pub use error::Error;
pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, clap::Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "stdinout-proxy")]
pub struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Cmd {
    /// Run server that serves the API
    Server(server::Server),
}

impl Cli {
    pub async fn run(self) -> Result {
        match self.cmd {
            Cmd::Server(server) => server.run().await,
        }
    }
}

#[tokio::main]
async fn main() -> Result {
    use clap::Parser;
    let cli = Cli::parse();
    if let Err(e) = cli.run().await {
        println!("Error: {}", e);
        std::process::exit(1);
    }
    Ok(())
}

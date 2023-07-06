use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use tracing::metadata::LevelFilter;
use url::Url;

mod client;
mod common;
mod secret;
mod server;

#[derive(Parser)]
#[command(name = "Sharebase")]
#[command(author = "Baasit. <bassit@onboardbase.com>")]
#[command(version = "0.0.1")]
#[command(about = "Share secrets securely through the terminal", long_about = None)]
pub struct Cli {
    ///Host and port URL denoting the receiver
    url: Option<Url>,
    /// Separated list of secrets to share. Key-Value pair is seperated by a comma. "my_key,my_value"
    #[arg(long, short)]
    secret: Vec<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ///Starts a UDP server to listen for shared secrets
    Receive {
        ///Sets the port to listen on
        #[arg(short, long = "listen", default_value = "[::1]:4433")]
        listen: SocketAddr,
    },
}

#[tokio::main]
async fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .pretty()
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(LevelFilter::DEBUG.into())
                    .from_env_lossy(),
            )
            .finish(),
    )
    .unwrap();

    let opt = Cli::parse();
    let code = {
        match opt.command {
            None => match client::run(opt).await {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    1
                }
            },
            Some(Commands::Receive { listen: _ }) => match server::run().await {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    1
                }
            },
        }
    };
    ::std::process::exit(code);
}

use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use tracing::metadata::LevelFilter;

mod common;
mod server;

#[derive(Parser)]
#[command(name = "Sharebase")]
#[command(author = "Baasit. <bassit@onboardbase.com>")]
#[command(version = "0.0.1")]
#[command(about = "Share secrets securely through the terminal", long_about = None)]
pub struct Cli {
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
            None => 1,
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

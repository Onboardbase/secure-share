use clap::Parser;
use libp2p::PeerId;
use std::str::FromStr;
use tracing::{error, metadata::LevelFilter};

mod hole_puncher;
mod secret;

#[derive(Parser)]
#[command(name = "Sharebase")]
#[command(author = "Baasit. <bassit@onboardbase.com>")]
#[command(version = "0.0.1")]
#[command(about = "Share secrets securely through the terminal", long_about = None)]
pub struct Cli {
    // /// Separated list of secrets to share. Key-Value pair is seperated by a comma. "my_key,my_value"
    // #[arg(long, short)]
    // secret: Vec<String>,
    /// The mode (share secrets, or rceive secrets).
    mode: Mode,

    /// Peer ID of the remote to send secrets to.
    #[clap(long)]
    remote_peer_id: Option<PeerId>,

    ///Port to establish connection on
    #[clap(long, short)]
    port: Option<i32>
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum Mode {
    Receive,
    Send,
}

impl FromStr for Mode {
    type Err = String;
    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "send" => Ok(Mode::Send),
            "receive" => Ok(Mode::Receive),
            _ => Err("Expected either 'send' or 'receive'".to_string()),
        }
    }
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

    let opts = Cli::parse();
    let code = {
        match hole_puncher::punch(opts) {
            Ok(_) => 1,
            Err(err) => {
                error!("{:#?}", err.to_string());
                1
            }
        }
    };
    ::std::process::exit(code);
}

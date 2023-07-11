use clap::Parser;
use libp2p::PeerId;
use std::str::FromStr;
use tracing::error;

mod hole_puncher;
mod logger;
mod secret;

#[derive(Parser, Debug)]
#[command(name = "share")]
#[command(author = "Wokebuild. <woke.build>")]
#[command(version = "0.0.7")]
#[command(about = "Share anything with teammates across machines via CLI.", long_about = None)]
pub struct Cli {
    /// Separated list of secrets to share. Key-Value pair is seperated by a comma. "my_key,my_value"
    #[arg(long, short)]
    secret: Option<Vec<String>>,

    /// The mode (share secrets, or receive secrets).
    mode: Mode,

    /// Peer ID of the remote to send secrets to.
    #[clap(long)]
    remote_peer_id: Option<PeerId>,

    ///Port to establish connection on
    #[clap(long, short)]
    port: Option<i32>,

    /// Specify if all logs should be displayed
    #[arg(long, default_value = "false")]
    verbose: bool,
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
    let opts = Cli::parse();
    logger::log(&opts).unwrap();

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

use clap::Parser;
use config::Config;
use libp2p::PeerId;
use std::{process::exit, str::FromStr};
use tracing::error;

mod config;
mod hole_puncher;
mod item;
mod logger;

#[derive(Parser, Debug)]
#[command(name = "share")]
#[command(author = "Wokebuild. <woke.build>")]
#[command(version = "0.0.11")]
#[command(about = "Share anything with teammates across machines via CLI.", long_about = None)]
pub struct Cli {
    /// Separated list of secrets to share. Key-Value pair is seperated by a comma. "my_key,my_value"
    #[arg(long, short)]
    secret: Option<Vec<String>>,

    /// List of messages or a message string to deliver to the receiver.
    /// e,g -m "Hi there" -m "See me"
    #[arg(long, short)]
    message: Option<Vec<String>>,

    /// List of file paths of files to deliver to the receiver.
    /// e,g -f "/path/to/file1" -f "../path/to/file2"
    #[arg(long, short)]
    file: Option<Vec<String>>,

    /// The mode (send secrets, or receive secrets).
    /// e,g `share send` or `share receive`
    mode: Mode,

    /// Peer ID of the remote to send secrets to.
    #[clap(long, short)]
    remote_peer_id: Option<PeerId>,

    ///Port to establish connection on
    #[clap(long, short)]
    port: Option<i32>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Configuration file for `share`
    #[arg(short, long)]
    config: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Parser, Copy)]
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
    let (mode, remote_peer_id, config) = match Config::new(&opts) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("{}", err);
            exit(1)
        }
    };

    logger::log(&config).unwrap();

    let code = {
        match hole_puncher::punch(mode, remote_peer_id, config) {
            Ok(_) => 1,
            Err(err) => {
                error!("{:#?}", err.to_string());
                1
            }
        }
    };
    ::std::process::exit(code);
}

use clap::Parser;
use config::Config;
use database::Store;
use libp2p::PeerId;
use network::punch;
use std::{process::exit, str::FromStr};
use tracing::error;

mod config;
mod database;
mod handlers;
mod item;
mod logger;
mod network;

#[derive(Parser, Debug)]
#[command(name = "scs")]
#[command(author = "Onboardbase. <onboardbase.com>")]
#[command(version = "0.1.3")]
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
    /// e,g `scs send` or `scs receive`
    mode: Mode,

    /// Peer ID of the remote to send secrets to.
    #[clap(long, short)]
    remote_peer_id: Option<PeerId>,

    // Name of the saved recipient to send a secret to.
    #[clap(long, short)]
    name: Option<String>,

    ///Port to establish connection on
    #[clap(long, short)]
    port: Option<i32>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Configuration file for `scs`
    #[arg(short, long)]
    config: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Parser, Copy)]
pub enum Mode {
    Receive,
    Send,
    List,
}

impl FromStr for Mode {
    type Err = String;
    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "send" => Ok(Mode::Send),
            "receive" => Ok(Mode::Receive),
            "list" => Ok(Mode::List),
            _ => Err("Expected either 'send' or 'receive' or 'list'".to_string()),
        }
    }
}

#[tokio::main]
async fn main() {
    let opts = Cli::parse();
    logger::log(&opts).unwrap();

    let store = match Store::initialize(None) {
        Ok(store) => store,
        Err(err) => {
            error!("{:#?}", err.to_string());
            exit(1)
        }
    };

    let (mode, remote_peer_id, config) = match Config::new(&opts, &store) {
        Ok(res) => res,
        Err(err) => {
            error!("{}", err);
            exit(1)
        }
    };

    let code = {
        match punch(mode, remote_peer_id, config, store) {
            Ok(_) => 1,
            Err(err) => {
                error!("{:#?}", err.to_string());
                1
            }
        }
    };
    ::std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use libp2p::PeerId;

    use crate::{Cli, Mode};

    #[test]
    fn cli() {
        let secret = None;
        let file = None;
        let message = None;
        let mode = Mode::Send;
        let remote_peer_id = Some(PeerId::random());
        let name = None;
        let debug = 0;
        let port = Some(5555);
        let config = None;

        let cli = Cli {
            secret,
            message,
            file,
            mode,
            remote_peer_id,
            debug,
            port,
            config,
            name,
        };

        assert_eq!(cli.debug, 0);
        assert_eq!(cli.file, None);
        assert!(cli.message.is_none());
        assert_ne!(cli.config, Some("path/to/config".to_string()))
    }
}

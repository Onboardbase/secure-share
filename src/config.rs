use std::{fs::OpenOptions, path::Path};

use anyhow::Result;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};

use crate::{item::Secret, Cli, Mode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    secret: Option<Vec<Secret>>,
    message: Option<Vec<String>>,
    file: Option<Vec<String>>,
    port: i32,
    debug: u8,
}

impl Config {
    fn from_cli(opts: &Cli) -> Config {
        let secrets = match &opts.secret {
            Some(scs) => {
                let secrets = scs.iter().map(Secret::from).collect::<Vec<_>>();
                Some(secrets)
            }
            None => None,
        };

        Config {
            secret: secrets,
            message: opts.message.clone(),
            file: opts.file.clone(),
            port: opts.port.unwrap_or(0),
            debug: opts.debug,
        }
    }

    fn from_config_file(path: String) -> Result<Config> {
        let config_file_path = Path::new(&path);
        let file = OpenOptions::new().read(true).open(config_file_path)?;
        let config: Config = serde_yaml::from_reader(file)?;
        Ok(config)
    }

    pub fn new(opts: &Cli) -> Result<(Mode, Option<PeerId>, Config)> {
        match &opts.config {
            None => {
                let config = Config::from_cli(opts);
                Ok((opts.mode, opts.remote_peer_id, config))
            }
            Some(path) => {
                let config = Config::from_config_file(path.to_string())?;
                Ok((opts.mode, opts.remote_peer_id, config))
            }
        }
    }

    pub fn verbose(&self) -> bool {
        !matches!(&self.debug, 0)
    }

    pub fn port(&self) -> i32 {
        self.port
    }

    pub fn file(&self) -> Option<Vec<String>> {
        self.file.clone()
    }

    pub fn message(&self) -> Option<Vec<String>> {
        self.message.clone()
    }

    pub fn secret(&self) -> Option<Vec<Secret>> {
        self.secret.clone()
    }
}

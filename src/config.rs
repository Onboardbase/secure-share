use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
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
    save_path: PathBuf,
    whitelists: Option<HashSet<Ipv4Addr>>,
    blacklists: Option<HashSet<Ipv4Addr>>,
}

impl Config {
    fn create_default_path() -> Result<PathBuf> {
        let dirs =
            directories_next::ProjectDirs::from("com", "onboardbase", "secureshare").unwrap();
        let path = dirs.data_local_dir();
        fs::create_dir_all(path).context("Failed to create default directory")?;
        Ok(PathBuf::from(path))
    }

    fn from_cli(opts: &Cli) -> Result<Config> {
        let secrets = match &opts.secret {
            Some(scs) => {
                let secrets = scs.iter().map(Secret::from).collect::<Vec<_>>();
                Some(secrets)
            }
            None => None,
        };

        let config = Config {
            secret: secrets,
            message: opts.message.clone(),
            file: opts.file.clone(),
            port: opts.port.unwrap_or(0),
            debug: opts.debug,
            save_path: Config::create_default_path()?,
            whitelists: None,
            blacklists: None,
        };
        Ok(config)
    }

    fn from_config_file(path: String) -> Result<Config> {
        let config_file_path = Path::new(&path);
        let file = OpenOptions::new().read(true).open(config_file_path)?;
        let config = serde_yaml::from_reader(file).map(|mut config: Config| {
            match config.save_path.to_str().unwrap() {
                "default" => config.save_path = Config::create_default_path().unwrap(),
                _ => {
                    let path = config.save_path.clone();
                    fs::create_dir_all(path)
                        .context(
                            "Failed to create 'save_path' directory. Please check your config file",
                        )
                        .unwrap();
                }
            }
            config
        })?;

        Ok(config)
    }

    pub fn new(opts: &Cli) -> Result<(Mode, Option<PeerId>, Config)> {
        match &opts.config {
            None => {
                let config = Config::from_cli(opts)?;
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

    pub fn save_path(&self) -> PathBuf {
        self.save_path.clone()
    }

    pub fn whitelists(&self) -> Option<HashSet<Ipv4Addr>> {
        self.whitelists.clone()
    }

    pub fn blacklists(&self) -> Option<HashSet<Ipv4Addr>> {
        self.blacklists.clone()
    }
}

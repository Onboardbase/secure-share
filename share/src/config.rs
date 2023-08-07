#![allow(dead_code)]

use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    net::Ipv4Addr,
    path::{Path, PathBuf},
    process::exit,
};

use anyhow::{anyhow, Context, Result};
use libp2p::PeerId;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    database::{peer::ScsPeer, Store},
    item::Secret,
    Cli, Mode,
};

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
    seed: String,
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
            seed: rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect(),
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

    fn remote_peer_id_polyfill(opts: &Cli, store: &Store) -> Result<PeerId> {
        let rpm = match &opts.remote_peer_id {
            Some(rpm) => *rpm,
            None => {
                let name = match &opts.name {
                    Some(name) => name,
                    None => {
                        return Err(anyhow!("Either a remote peer id or a name must be present"))
                    }
                };
                let peer = ScsPeer::get_by_name(name.to_string(), store)?;
                peer.peer_id()?
            }
        };
        Ok(rpm)
    }

    fn list_all_saved_peers(store: &Store) -> Result<Vec<ScsPeer>> {
        ScsPeer::fetch_all_peers(store)
    }

    pub fn new(opts: &Cli, store: &Store) -> Result<(Mode, Option<PeerId>, Config)> {
        if opts.mode == Mode::List {
            let peers = Self::list_all_saved_peers(store)?;
            if peers.is_empty() {
                println!("No saved peer");
            } else {
                for peer in peers {
                    println!("- {}", peer.name());
                }
            }
            exit(1)
        }

        let rpm = match &opts.mode {
            Mode::Send => Some(Self::remote_peer_id_polyfill(opts, store)?),
            Mode::Receive => None,
            Mode::List => exit(1),
        };

        match &opts.config {
            None => {
                let config = Config::from_cli(opts)?;
                Ok((opts.mode, rpm, config))
            }
            Some(path) => {
                let config = Config::from_config_file(path.to_string())?;
                Ok((opts.mode, rpm, config))
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

    fn pad_seed_key(&self, mut s: String) -> String {
        while s.len() < 32 {
            s.push(' ');
        }
        s.truncate(32);
        s
    }

    pub fn seed_key(&self) -> String {
        let seed = self.seed.clone();
        self.pad_seed_key(seed)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{database::Store, item::Secret, Cli, Mode};

    use super::Config;
    use anyhow::{Ok, Result};
    use assert_fs::prelude::FileWriteStr;
    use libp2p::PeerId;

    #[test]
    fn default_path_created() -> Result<()> {
        let path = Config::create_default_path()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn config_from_cli() -> Result<()> {
        let secret = None;
        let file = None;
        let message = Some(vec!["from cli".to_string()]);
        let mode = Mode::Send;
        let peer_id = PeerId::random();
        let name = None;
        let debug = 0;
        let port = Some(5555);
        let config = None;

        let opts = Cli {
            secret,
            message,
            file,
            mode,
            remote_peer_id: Some(peer_id),
            debug,
            port,
            config,
            name,
        };
        let db_path = assert_fs::NamedTempFile::new("scs.db3")?;
        let store = Store::initialize(Some(db_path.path().to_path_buf()))?;

        let config = Config::new(&opts, &store)?;

        assert_eq!(config.0, Mode::Send);
        assert_eq!(config.1, Some(peer_id));
        assert_eq!(config.2.port(), 5555);

        db_path.close()?;
        Ok(())
    }

    fn make_config() -> Result<Config> {
        let secret = vec![Secret::from("hi,there".to_string())];
        let message = vec!["new message".to_string()];

        let test_file = assert_fs::NamedTempFile::new("sample.txt")?;
        test_file.write_str("A test\nActual content\nMore content\nAnother test")?;
        let file = vec![test_file.path().to_str().unwrap().to_string()];

        let config = Config {
            secret: Some(secret),
            message: Some(message),
            file: Some(file),
            port: 5555,
            debug: 1,
            save_path: PathBuf::from(test_file.parent().unwrap()),
            whitelists: None,
            blacklists: None,
            seed: "test".to_string(),
        };
        Ok(config)
    }

    #[test]
    fn config_file() -> Result<()> {
        let yaml_config = format!(
            "
            port: 5555 
            save_path: 'default'
            secret:
            - key: foo
              value: bar
            - key: baz
              value: woo
            message: 
            - new message from me
            - test message
            debug: 1
            seed: test
        "
        );
        let file = assert_fs::NamedTempFile::new("config.yml")?;
        file.write_str(&yaml_config)?;
        let config = Config::from_config_file(file.path().to_str().unwrap().to_string())?;
        assert_eq!(config.port(), 5555);
        let project_dir =
            directories_next::ProjectDirs::from("com", "onboardbase", "secureshare").unwrap();
        let path = project_dir.data_local_dir();

        assert!(path.exists());
        assert_eq!(config.save_path(), PathBuf::from(path));
        assert_eq!(config.seed.len(), 4);

        file.close()?;
        Ok(())
    }

    #[test]
    fn fail_to_polyfill_remote_peer_id() -> Result<()> {
        let secret = None;
        let file = None;
        let message = Some(vec!["from cli".to_string()]);
        let mode = Mode::Send;
        // let peer_id = PeerId::random();
        let name = None;
        let debug = 0;
        let port = Some(5555);
        let config = None;

        let opts = Cli {
            secret,
            message,
            file,
            mode,
            remote_peer_id: None,
            debug,
            port,
            config,
            name,
        };

        let db_path = assert_fs::NamedTempFile::new("scs.db3")?;
        let store = Store::initialize(Some(db_path.path().to_path_buf()))?;

        let rpm = Config::remote_peer_id_polyfill(&opts, &store);
        assert!(rpm.is_err());
        let _ = rpm.map_err(|err| {
            assert!(err
                .to_string()
                .contains("Either a remote peer id or a name must be present"))
        });

        db_path.close()?;
        Ok(())
    }

    #[test]
    fn file_to_be_sent() -> Result<()> {
        let config = make_config();
        assert!(config.is_ok());

        let config = config?;
        let files = config.file();
        assert!(files.is_some());

        let files = files.unwrap();
        assert_eq!(files.len(), 1);

        for file in files {
            let path = PathBuf::from(file);
            assert!(!path.exists());
            // let content = fs::read_to_string(path)?;
            // assert_eq!(content, "A test\nActual content\nMore content\nAnother test".to_string());
        }
        Ok(())
    }

    #[test]
    fn messages() -> Result<()> {
        let config = make_config()?;
        assert!(config.message().is_some());

        let binding = config.message().unwrap();
        let msg = binding.first().unwrap();
        assert_eq!(msg, "new message");
        Ok(())
    }

    #[test]
    fn secrets() -> Result<()> {
        let config = make_config()?;
        assert!(config.secret().is_some());

        let binding = config.secret().unwrap();
        let secret = binding.first().unwrap();
        assert_eq!(secret.key, "hi");
        assert_ne!(secret.value, "hi");
        Ok(())
    }

    #[test]
    fn verbose() -> Result<()> {
        let config = make_config()?;
        assert!(config.verbose());
        Ok(())
    }

    #[test]
    fn lists() -> Result<()> {
        let config = make_config()?;
        assert_eq!(config.blacklists(), None);
        assert_eq!(config.whitelists(), None);
        Ok(())
    }

    #[test]
    fn seed() -> Result<()> {
        let seed_key = "greyhounds";

        let yaml_config = format!(
            "
            port: 5555 
            save_path: 'default'
            secret:
            - key: foo
              value: bar
            - key: baz
              value: woo
            message: 
            - new message from me
            - test message
            debug: 1
            seed: {seed_key}
        "
        );
        let config: Config = serde_yaml::from_str(&yaml_config)?;

        let padded_string = config.seed_key();
        let padded_string = &padded_string[seed_key.len()..padded_string.len()];
        assert_eq!(padded_string.len(), (32 - seed_key.len()));
        Ok(())
    }

    #[test]
    fn pad_string() -> Result<()> {
        let s = "hi".to_string();
        let config = make_config()?;
        let padded_str = config.pad_seed_key(s.clone());

        assert_eq!(padded_str.len(), 32);
        assert_ne!(s, padded_str);
        assert!(padded_str.contains(&s));

        Ok(())
    }
}

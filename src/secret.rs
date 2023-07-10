use std::{
    cmp::Ordering,
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Secret {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecretResponse {
    pub status: Status,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Status {
    FAILED,
    SUCCESS,
}

impl Secret {
    pub fn secrets_from_string(secrets: Vec<String>) -> Vec<Secret> {
        secrets
            .into_iter()
            .filter_map(|secret| Secret::secret_from_string(secret).ok())
            .collect::<Vec<_>>()
    }

    fn secret_from_string(secret: String) -> Result<Secret> {
        let key_value_pair = secret.split(',').collect::<Vec<_>>();
        if key_value_pair.len() > 2 {
            return Err(anyhow!("Key Value Pair for secret is incompatible"));
        }
        if key_value_pair.len() < 2 {
            return Err(anyhow!("Key or Value not found for {secret}"));
        }
        let secret = Secret {
            key: key_value_pair.first().unwrap().to_string(),
            value: key_value_pair.get(1).unwrap().to_string(),
        };
        Ok(secret)
    }

    pub fn validate_secrets(secrets: Vec<String>) -> Result<Vec<String>> {
        let res = secrets
            .into_iter()
            .map(|secret| {
                let value_key: Vec<&str> = secret.split(',').collect();
                match value_key.len().cmp(&"2".parse::<usize>().unwrap()) {
                    Ordering::Equal => secret,
                    Ordering::Greater => anyhow!(
                        "Secret must contain just key and value. {} violates that rule",
                        secret
                    )
                    .to_string(),
                    Ordering::Less => {
                        anyhow!("Secret cannot be empty. Please check your arguments").to_string()
                    }
                }
            })
            .collect::<Vec<_>>();
        Ok(res)
    }

    pub fn bulk_secrets_save(secrets: Vec<Secret>) -> Result<()> {
        let dirs = directories_next::ProjectDirs::from("build", "woke", "share").unwrap();
        let path = dirs.data_local_dir();
        let secret_default_path = path.join("secrets.json");
        let secrets_file =
            File::create(secret_default_path).context("Failed to open secrets file storage")?;
        let mut writer = BufWriter::new(secrets_file);
        serde_json::to_writer(&mut writer, &secrets)?;
        writer.flush().context("Failed to save secrets")?;
        Ok(())
    }
}

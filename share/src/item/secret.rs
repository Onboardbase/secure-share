use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter, Write},
    path::Path,
    process::exit,
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tracing::error;

use super::Secret;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecretResponse {
    pub status: Status,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Status {
    Failed,
    Succes,
}

impl Secret {
    pub fn secret_from_string(secret: String) -> Result<Secret> {
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

    pub fn save_secret(&self, path: &Path) -> Result<()> {
        let secret_default_path = path.join("secrets.json");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(secret_default_path)?;

        let reader = BufReader::new(file.try_clone()?);
        //in case the file was just created and it is empty, automatically add a column and then push the result
        let empty: Result<Vec<Secret>> = Ok(vec![]);
        let mut contents: Vec<Secret> = serde_json::from_reader(reader).or(empty)?;
        contents.push(self.clone());

        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &contents)?;
        writer.flush().context("Failed to save secrets")?;

        Ok(())
    }
}

impl From<String> for Secret {
    fn from(secret: String) -> Secret {
        match Secret::secret_from_string(secret) {
            Ok(secret) => secret,
            Err(err) => {
                error!("{}", err.to_string());
                exit(1)
            }
        }
    }
}

impl From<&String> for Secret {
    fn from(secret: &String) -> Secret {
        match Secret::secret_from_string(secret.to_string()) {
            Ok(secret) => secret,
            Err(err) => {
                error!("{}", err.to_string());
                exit(1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Secret;
    use anyhow::Result;
    use assert_fs::prelude::{PathAssert, PathChild};
    use predicates::prelude::*;

    #[test]
    fn column_delimited_secret() -> Result<()> {
        let secret = Secret::secret_from_string("hi:there".to_string());
        assert!(secret.is_err());
        let _ =
            secret.map_err(|err| assert!(err.to_string().contains("Key or Value not found for")));
        Ok(())
    }

    #[test]
    fn secret() -> Result<()> {
        let secret = Secret::from("hi,there".to_string());
        assert_eq!(secret.key, "hi");
        assert_eq!(secret.value, "there");
        Ok(())
    }

    #[test]
    fn save_secret() -> Result<()> {
        let save_dir = assert_fs::TempDir::new()?;
        let secret = Secret::from("foo,bar".to_string());
        secret.save_secret(&save_dir.path())?;

        let secret_file = save_dir.child("secrets.json");
        secret_file.assert(predicate::path::exists());

        let secrets = r#"[{"key":"foo","value":"bar"}]"#;
        secret_file.assert(predicate::str::contains(secrets));

        save_dir.close()?;
        Ok(())
    }
}

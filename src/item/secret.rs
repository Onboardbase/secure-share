use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter, Write},
    path::Path,
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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

    // pub fn validate_secrets(secrets: Vec<String>) -> Result<Vec<String>> {
    //     let res = secrets
    //         .into_iter()
    //         .map(|secret| {
    //             let value_key: Vec<&str> = secret.split(',').collect();
    //             match value_key.len().cmp(&"2".parse::<usize>().unwrap()) {
    //                 Ordering::Equal => secret,
    //                 Ordering::Greater => anyhow!(
    //                     "Secret must contain just key and value. {} violates that rule",
    //                     secret
    //                 )
    //                 .to_string(),
    //                 Ordering::Less => {
    //                     anyhow!("Secret cannot be empty. Please check your arguments").to_string()
    //                 }
    //             }
    //         })
    //         .collect::<Vec<_>>();
    //     Ok(res)
    // }

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

    // pub fn bulk_secrets_save(secrets: Vec<Secret>) -> Result<()> {
    //     let dirs = directories_next::ProjectDirs::from("build", "woke", "wokeshare").unwrap();
    //     let path = dirs.data_local_dir();
    //     fs::create_dir_all(path).context("Failed to create secrets directory")?;
    //     let secret_default_path = path.join("secrets.json");
    //     let secrets_file = File::create(&secret_default_path).context(format!(
    //         "Failed to open secrets file storage at: {:?}",
    //         secret_default_path
    //     ))?;
    //     let mut writer = BufWriter::new(secrets_file);
    //     serde_json::to_writer(&mut writer, &secrets)?;
    //     writer.flush().context("Failed to save secrets")?;
    //     Ok(())
    // }
}

use std::{ffi::OsString, fs};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

mod item_file;
mod item_message;
mod secret;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    item_type: ItemType,
    message: Option<item_message::ItemMessage>,
    secret: Option<secret::Secret>,
    file: Option<item_file::ItemFile>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ItemType {
    File,
    Message,
    Secret,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemResponse {
    pub status: Status,
    pub no_of_success: usize,
    pub no_of_fails: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Status {
    Failed,
    Succes,
}

impl Item {
    pub fn new(param: String, item_type: ItemType) -> Result<Self> {
        let item = match item_type {
            ItemType::Secret => {
                let secret = secret::Secret::secret_from_string(param)?;
                Item {
                    item_type,
                    secret: Some(secret),
                    message: None,
                    file: None,
                }
            }
            ItemType::File => {
                let file = item_file::ItemFile::new(OsString::from(param))?;
                Item {
                    item_type,
                    secret: None,
                    message: None,
                    file: Some(file),
                }
            }
            ItemType::Message => Item {
                item_type,
                message: Some(item_message::ItemMessage::new(param)),
                secret: None,
                file: None,
            },
        };

        Ok(item)
    }

    pub fn save(&self) -> Result<()> {
        let dirs = directories_next::ProjectDirs::from("build", "woke", "wokeshare").unwrap();
        let path = dirs.data_local_dir();
        fs::create_dir_all(path).context("Failed to create secrets directory")?;
        match self.item_type {
            ItemType::File => self.file.clone().unwrap().save(path)?,
            ItemType::Message => self.message.clone().unwrap().save(path)?,
            ItemType::Secret => self.secret.clone().unwrap().save_secret(path)?,
        }
        Ok(())
    }

    pub fn item_type(&self) -> ItemType {
        self.item_type.clone()
    }
}

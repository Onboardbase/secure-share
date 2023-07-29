use std::ffi::OsString;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config::Config;

mod item_file;
mod item_message;
mod secret;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Secret {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
//TODO implemnt fmt::Display for all
pub struct Item {
    item_type: ItemType,
    message: Option<item_message::ItemMessage>,
    secret: Option<Secret>,
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
    pub err: Option<String>,
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
                let secret = Secret::secret_from_string(param)?;
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

    pub fn save(&self, config: &Config) -> Result<()> {
        let path = &config.save_path();
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

impl From<Secret> for Item {
    fn from(secret: Secret) -> Item {
        Item {
            item_type: ItemType::Secret,
            secret: Some(secret),
            message: None,
            file: None,
        }
    }
}

impl From<&Secret> for Item {
    fn from(secret: &Secret) -> Item {
        Item {
            item_type: ItemType::Secret,
            secret: Some(secret.clone()),
            message: None,
            file: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    use super::{Item, ItemType};
    use anyhow::Result;
    use assert_fs::prelude::{FileWriteStr, PathAssert, PathChild};
    use predicates::prelude::*;

    fn make_config(path: &str) -> Result<Config> {
        let yaml_config = format!(
            "
            port: 5555 
            save_path: '{path}'
            secret:
            - key: foo
              value: bar
            - key: baz
              value: woo
            message: 
            - new message from me
            - test message
            debug: 1
        "
        );
        let config: Config = serde_yaml::from_str(&yaml_config)?;

        Ok(config)
    }

    #[test]
    fn secret_item() -> Result<()> {
        let item = Item::new("hi, there".to_string(), ItemType::Secret);
        assert!(item.is_ok());

        let item = item?;
        assert_eq!(item.item_type(), ItemType::Secret);

        let save_dir = assert_fs::TempDir::new()?;
        let config = make_config(save_dir.path().to_str().unwrap())?;
        item.save(&config)?;

        save_dir
            .child("messages.txt")
            .assert(predicate::path::missing());

        let secret_file = save_dir.child("secrets.json");
        secret_file.assert(predicate::path::exists());

        let secrets = r#"[{"key":"hi","value":" there"}]"#;
        secret_file.assert(predicate::str::contains(secrets));

        save_dir.close()?;
        Ok(())
    }

    #[test]
    fn message_item() -> Result<()> {
        let item = Item::new(
            "new message and another one too".to_string(),
            ItemType::Message,
        );
        assert!(item.is_ok());

        let item = item?;
        assert_eq!(item.item_type(), ItemType::Message);

        let save_dir = assert_fs::TempDir::new()?;
        let config = make_config(save_dir.path().to_str().unwrap())?;
        item.save(&config)?;

        save_dir
            .child("secrets.json")
            .assert(predicate::path::missing());

        let message_file = save_dir.child("messages.txt");
        message_file.assert(predicate::path::exists());
        message_file.assert(predicate::str::contains("new message and another one too"));

        save_dir.close()?;
        Ok(())
    }

    #[test]
    fn file_item() -> Result<()> {
        let test_file = assert_fs::NamedTempFile::new("sample.txt")?;
        test_file.write_str("A test\nActual content\nMore content\nAnother test")?;

        let item = Item::new(test_file.to_str().unwrap().to_string(), ItemType::File);
        assert!(item.is_ok());

        let item = item?;
        assert_eq!(item.item_type(), ItemType::File);

        let save_dir = assert_fs::TempDir::new()?;
        let config = make_config(save_dir.path().to_str().unwrap())?;
        item.save(&config)?;
        test_file.close()?;

        let saved_file = save_dir.child("sample.txt");
        saved_file.assert(predicate::path::exists());
        saved_file.assert(predicate::str::contains(
            "A test\nActual content\nMore content\nAnother test",
        ));

        save_dir.close()?;
        Ok(())
    }
}

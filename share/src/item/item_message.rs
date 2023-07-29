use std::{fs::OpenOptions, io::Write, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ItemMessage {
    msg: String,
}

impl ItemMessage {
    pub fn new(msg: String) -> ItemMessage {
        ItemMessage { msg }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let message_file_path = path.join("messages.txt");
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(message_file_path)?;

        let message = &self.msg;
        let message = if message.ends_with('\n') {
            message.clone()
        } else {
            format!("{message}\n")
        };
        file.write_all(message.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ItemMessage;
    use anyhow::Result;
    use assert_fs::prelude::{PathAssert, PathChild};
    use predicates::prelude::*;

    #[test]
    fn new_message() {
        let item = ItemMessage::new("hi there".to_string());
        assert_eq!("hi there", item.msg);
    }

    #[test]
    fn save_message() -> Result<()> {
        let dir = assert_fs::TempDir::new()?;
        let lorem_ipsum = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. 
        Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, 
        when an unknown printer took a galley of type and scrambled it to make a type specimen book. 
        It has survived not only five centuries";

        let item = ItemMessage::new(lorem_ipsum.into());
        item.save(dir.path())?;

        let message_file = dir.child("messages.txt");
        message_file.assert(predicate::path::exists());
        message_file.assert(predicate::str::is_match(format!("{lorem_ipsum}\n"))?);
        dir.close()?;

        Ok(())
    }
}

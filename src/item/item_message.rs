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

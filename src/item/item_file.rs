use anyhow::{anyhow, Result};
use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    u8,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ItemFile {
    name: OsString,
    path: OsString,
    data: Vec<u8>,
    extension: OsString,
}

impl ItemFile {
    pub fn new(file_path: OsString) -> Result<ItemFile> {
        let path = Path::new(&file_path);

        if !path.exists() {
            return Err(anyhow!("File at {:?} does not exist", file_path));
        }

        if path.is_dir() {
            return Err(anyhow!(
                "File at {:?} is a directory, not a file",
                file_path
            ));
        }
        let file_name = path.file_name().unwrap().to_os_string();
        let extension = path.extension().unwrap_or_default().to_os_string();

        let mut f = File::open(path)?;
        let mut data = vec![];
        f.read_to_end(&mut data)?;

        let item = ItemFile {
            name: file_name,
            path: path.into(),
            data,
            extension,
        };
        Ok(item)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let name = self.name.to_str().unwrap();
        let file_path = path.join(name);
        println!("{:?}", file_path);
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)?;
        file.write_all(&self.data)?;
        Ok(())
    }
}

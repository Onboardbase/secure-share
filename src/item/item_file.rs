use anyhow::{anyhow, Result};
use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    u8,
};
use tracing::error;

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
            error!("File at {:?} does not exist", file_path);
            return Err(anyhow!("File at {:?} does not exist", file_path));
        }

        if path.is_dir() {
            error!("File at {:?} is a directory, not a file", file_path);
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
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)?;
        file.write_all(&self.data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::ItemFile;
    use anyhow::Result;
    use assert_fs::prelude::{FileWriteStr, PathAssert, PathChild};
    use predicates::prelude::*;

    #[test]
    fn directory_file() -> Result<()> {
        let dir = assert_fs::TempDir::new()?;
        let item = ItemFile::new(OsString::from(dir.path()));
        assert!(item.is_err());
        let _ = item.map_err(|err| assert!(err.to_string().contains("is a directory")));
        dir.close()?;
        Ok(())
    }

    #[test]
    fn missing_file() -> Result<()> {
        let item = ItemFile::new(OsString::from("jhjsi/jkjsjdjia"));
        assert!(item.is_err());
        let _ = item.map_err(|err| assert!(err.to_string().contains("does not exist")));
        Ok(())
    }

    #[test]
    fn new_file() -> Result<()> {
        let file = assert_fs::NamedTempFile::new("foo.txt")?;
        let lorem_ipsum = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. 
        Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, 
        when an unknown printer took a galley of type and scrambled it to make a type specimen book. 
        It has survived not only five centuries";
        file.write_str(lorem_ipsum)?;

        let item = ItemFile::new(OsString::from(file.path()))?;
        assert_eq!(item.extension, OsString::from("txt"));
        assert_eq!(item.data, lorem_ipsum.as_bytes());

        file.close()?;
        Ok(())
    }

    #[test]
    fn save_file() -> Result<()> {
        let file = assert_fs::NamedTempFile::new("bar.txt")?;
        let dir = assert_fs::TempDir::new()?;
        let lorem_ipsum = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. 
        Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, 
        when an unknown printer took a galley of type and scrambled it to make a type specimen book. 
        It has survived not only five centuries";
        file.write_str(lorem_ipsum)?;

        let item = ItemFile::new(OsString::from(file.path()))?;
        item.save(dir.path())?;
        file.close()?;

        let saved_file = dir.child("bar.txt");
        saved_file.assert(predicate::path::exists());
        saved_file.assert(predicate::str::contains(lorem_ipsum));
        dir.close()?;

        Ok(())
    }
}

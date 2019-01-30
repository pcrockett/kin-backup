use super::fsutil;
use std::path::PathBuf;

pub struct BackupPackage {
    path: PathBuf
}

impl BackupPackage {

    pub fn from(path: &PathBuf) -> BackupPackage {
        BackupPackage {
            path: path.to_owned()
        }
    }

    pub fn init(path: &PathBuf) -> Result<BackupPackage, failure::Error> {
        fsutil::ensure_empty_dir(path)?;
        Ok(BackupPackage::from(path))
    }

    pub fn config_dir(&self) -> PathBuf {
        self.path.join(".kin")
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir().join("config.json")
    }

    pub fn public_archive(&self) -> PathBuf {
        self.path.join("public.zip")
    }

    pub fn private_archive(&self) -> PathBuf {
        self.path.join("private.kin")
    }
}
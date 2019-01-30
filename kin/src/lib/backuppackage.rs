use super::fsutil;
use std::fs::File;
use std::io::{ BufWriter, Write };
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

    pub fn init(path: &PathBuf, encrypted_keys: Vec<String>) -> Result<BackupPackage, failure::Error> {

        fsutil::ensure_empty_dir(path)?;
        let package = BackupPackage::from(path);
        fsutil::ensure_empty_dir(&package.config_dir())?;
        let settings = PackageSettings { encrypted_keys };
        settings.write(&package.config_file())?;
        Ok(package)
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

#[derive(Serialize, Deserialize)]
pub struct PackageSettings {
    pub encrypted_keys: Vec<String>
}

impl PackageSettings {

    pub fn write(&self, path: &PathBuf) -> Result<(), failure::Error> {

        let config_serialized = serde_json::to_string_pretty(self)?;

        let file = File::create(path)?;
        let mut file = BufWriter::new(file);
        file.write(config_serialized.as_bytes())?;
        file.flush()?;

        Ok(())
    }
}

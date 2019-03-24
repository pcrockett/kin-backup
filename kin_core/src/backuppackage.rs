use super::fsutil;
use super::libsodium::{ EncryptedMasterKey, MasterKey };
use super::Error;
use failure:: { bail };
use serde::{ Deserialize, Serialize };
use std::fs::File;
use std::iter::Iterator;
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

    pub fn init(path: &PathBuf, encrypted_keys: Vec<EncryptedMasterKey>) -> Result<BackupPackage, Error> {

        fsutil::ensure_empty_dir(path)?;
        let package = BackupPackage::from(path);
        fsutil::ensure_empty_dir(&package.config_dir_path())?;

        let keys = encrypted_keys.iter()
            .map(|x| EncryptedKey {
                data: x.data(),
                passphrase_salt: x.passphrase_salt(),
                nonce: x.nonce()
            })
            .collect();

        let settings = PackageSettings { encrypted_keys: keys };
        settings.write(&package.config_file_path())?;

        set_readonly(&package.config_file_path())?;

        Ok(package)
    }

    pub fn config_dir_path(&self) -> PathBuf {
        self.path.join(".kin")
    }

    pub fn config_file_path(&self) -> PathBuf {
        self.config_dir_path().join("config.json")
    }

    pub fn public_archive_path(&self) -> PathBuf {
        self.path.join("public.zip")
    }

    pub fn private_archive_path(&self) -> PathBuf {
        self.path.join("private.kin")
    }

    pub fn decrypt_exe_path(&self) -> PathBuf {
        if cfg!(target_os = "linux") {
            return self.path.join("decrypt");
        } else if cfg!(target_os = "windows") {
            return self.path.join("decrypt.exe");
        } else {
            panic!("only Linux and Windows are supported at this time.");
        }
    }

    pub fn readme_path(&self) -> PathBuf {
        self.path.join("readme.html")
    }

    pub fn decrypt_master_key(&self, passphrase: &String) -> Result<MasterKey, Error> {

        let settings = match PackageSettings::read(&self.config_file_path()) {
            Ok(settings) => settings,
            Err(err) => bail!("Unable to parse {}: {}", self.config_file_path().to_str().unwrap(), err)
        };

        let encrypted_keys: Vec<EncryptedMasterKey> = settings.encrypted_keys.iter()
            .map(|x| EncryptedMasterKey::new(&x.data, &x.passphrase_salt, &x.nonce).unwrap())
            .collect();

        for encr_key in encrypted_keys {

            match encr_key.decrypt(passphrase) {
                Ok(key) => return Ok(key),
                Err(_) => continue // Expected; check the next key in the collection
            };
        }

        bail!("Unable to decrypt master key"); // TODO: Be more specific
    }
}

#[derive(Serialize, Deserialize)]
pub struct PackageSettings {
    encrypted_keys: Vec<EncryptedKey>
}

#[derive(Serialize, Deserialize)]
struct EncryptedKey {
    data: String,
    passphrase_salt: String,
    nonce: String
}

impl PackageSettings {

    pub fn write(&self, path: &PathBuf) -> Result<(), Error> {

        let config_serialized = serde_json::to_string_pretty(self)?;

        let file = File::create(path)?;
        let mut file = BufWriter::new(file);
        file.write(config_serialized.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    pub fn read(path: &PathBuf) -> Result<PackageSettings, Error> {

        let file = File::open(path)?;
        let settings = serde_json::from_reader(file)?;
        Ok(settings)
    }
}

#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;

#[cfg(target_os = "linux")]
fn set_readonly(path: &PathBuf) -> Result<(), Error> {
    // Set read-only permissions on the config file for user, group, and others.
    let perms = PermissionsExt::from_mode(0o444);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn set_readonly(_path: &PathBuf) -> Result<(), Error> {
    // TODO: Set readonly flag on file
    Ok(())
}
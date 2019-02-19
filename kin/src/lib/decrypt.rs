use super::backuppackage::BackupPackage;
use super::cmdline::DecryptArgs;
use super::libsodium;
use super::libsodium::{ MasterKey };
use super::ui;
use failure::{ bail };
use log::{ info };
use std::fs::{ File, OpenOptions };
use std::path::PathBuf;

pub fn run(args: &DecryptArgs) -> Result<(), failure::Error> {

    let source_dir = match &args.backup_dir {
        Some(dir) => dir.to_owned(),
        None => std::env::current_dir()?
    };

    let backup_package = BackupPackage::from(&source_dir);

    let dest_archive = match &args.destination {
        Some(path) => path.to_owned(),
        None => prompt_dest_archive()?
    };

    info!("Extracting to {}", dest_archive.to_str().unwrap());

    let passphrase = rpassword::read_password_from_tty(Some("Enter passphrase: "))?;
    let master_key = backup_package.decrypt_master_key(&passphrase)?;

    decrypt_archive(&backup_package.private_archive_path(), &dest_archive, master_key)?;

    Ok(())
}

fn prompt_dest_archive() -> Result<PathBuf, failure::Error> {

    println!("Where do you want to save the decrypted .zip archive?");

    loop {

        let path = ui::prompt("Enter file path: ")?;
        if path.len() == 0 {
            continue;
        }

        let path = PathBuf::from(path);
        let dir = path.parent().unwrap();

        if dir.to_str().unwrap().len() == 0 {
            // User probably just entered a file name and expects it to go in
            // the current working directory
            return Ok(path);
        } else if dir.is_dir() {
            return Ok(path);
        } else {
            println!("{} doesn't exist.", dir.to_str().unwrap());
        }
    }
}

fn decrypt_archive(encrypted_archive_path: &PathBuf, dest_path: &PathBuf, master_key: MasterKey) -> Result<(), failure::Error> {

    let dest_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dest_path);

    let mut dest_file = match dest_file {
        Ok(file) => file,
        Err(err) => bail!("Unable to open {}: {}", dest_path.to_str().unwrap(), err)
    };

    let mut reader = match File::open(&encrypted_archive_path) {
        Ok(file) => file,
        Err(err) => bail!("Unable to open {}: {}", encrypted_archive_path.to_str().unwrap(), err)
    };

    libsodium::decrypt(&master_key, &mut reader, &mut dest_file)?;

    Ok(())
}

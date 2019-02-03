use super::backuppackage::BackupPackage;
use super::cmdline::DecryptArgs;
use super::libsodium::{ DecryptingWriter, MasterKey };
use failure::{ bail };
use log::{ info };
use std::fs::{ File, OpenOptions };
use std::io;
use std::io::Write;
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

    decrypt_archive(&backup_package.private_archive(), &dest_archive, master_key)?;

    Ok(())
}

fn prompt_dest_archive() -> Result<PathBuf, failure::Error> {

    println!("Where do you want to save the decrypted .zip archive?");

    loop {

        let path = prompt("Enter file path: ")?;
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

fn prompt(question: &str) -> Result<String, failure::Error> {

    print!("{} ", question);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    input.pop().unwrap(); // Remove newline at end

    Ok(input)
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

    let mut writer = DecryptingWriter::new(&master_key, &mut dest_file);

    let encrypted_archive_size = encrypted_archive_path.metadata()?.len();
    writer.consume(&mut reader, encrypted_archive_size)?;

    Ok(())
}

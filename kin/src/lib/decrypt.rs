use super::backuppackage::BackupPackage;
use super::cmdline::DecryptArgs;
use super::libsodium::{ DecryptingWriter, MasterKey };
use std::fs::{ File, OpenOptions };
use std::io;
use std::io::Write;
use std::path::PathBuf;

pub fn run(args: &DecryptArgs) -> Result<(), failure::Error> {

    let dest_dir = match &args.dest_dir {
        Some(dir) => dir.to_owned(),
        None => PathBuf::from(prompt("Enter destination directory: ")?)
    };

    let source_dir = match &args.backup_dir {
        Some(dir) => dir.to_owned(),
        None => std::env::current_dir()?
    };

    let password = rpassword::read_password_from_tty(Some("Enter password: "))?;
    let backup_package = BackupPackage::from(&source_dir);
    let master_key = backup_package.decrypt_master_key(&password)?;

    decrypt_archive(&backup_package.private_archive(), &dest_dir, master_key)?;

    Ok(())
}

fn prompt(question: &str) -> Result<String, failure::Error> {

    print!("{} ", question);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    input.pop().unwrap(); // Remove newline at end

    Ok(input)
}

fn decrypt_archive(archive_path: &PathBuf, dest_dir: &PathBuf, master_key: MasterKey) -> Result<(), failure::Error> {

    let dest_path = dest_dir.join("temp.zip");
    let mut dest_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dest_path)?;

    let mut reader = File::open(&archive_path)?;
    let mut writer = DecryptingWriter::new(&master_key, &mut dest_file);

    let encrypted_archive_size = archive_path.metadata()?.len();
    writer.consume(&mut reader, encrypted_archive_size)?;

    Ok(())
}

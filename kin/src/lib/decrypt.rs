use super::backuppackage::BackupPackage;
use super::cmdline::DecryptArgs;
use super::libsodium::MasterKey;
use std::path::PathBuf;

pub fn run(args: &DecryptArgs) -> Result<(), failure::Error> {

    let dest_dir = match &args.dest_dir {
        Some(dir) => dir.to_owned(),
        None => panic!("TODO: Prompt user for a destination directory")
    };

    let source_dir = match &args.backup_dir {
        Some(dir) => dir.to_owned(),
        None => std::env::current_dir()?
    };

    let password = String::from("TODO: Prompt user for password");
    let backup_package = BackupPackage::from(&source_dir);
    let master_key = backup_package.decrypt_master_key(&password)?;

    decrypt_archive(&backup_package.private_archive(), &dest_dir, master_key)?;

    Ok(())
}

fn decrypt_archive(archive_path: &PathBuf, dest_dir: &PathBuf, master_key: MasterKey) -> Result<(), failure::Error> {
    panic!("not implemented yet");
}

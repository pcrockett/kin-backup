use super::cmdline::CompileArgs;
use super::fsutil;
use super::kinproject::KinProject;
use super::kinsettings::KinSettings;
use log::{ info };
use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::secretbox::xsalsa20poly1305::{ Key, Nonce};
use std::fs;
use std::path::PathBuf;

struct NonceRecord {
    file: String,
    nonce: Nonce
}

pub fn run(args: &CompileArgs) -> Result<(), failure::Error> {

    fsutil::ensure_empty_dir(&args.dest_dir)?;

    let project = match &args.project_dir {
        Some(dir) => KinProject::from(&dir),
        None => KinProject::from(&std::env::current_dir()?)
    };

    copy_public_dir(&project, &args.dest_dir)?;
    copy_private_dir(&project, &args)?;

    Ok(())
}

fn copy_private_dir(project: &KinProject, args: &CompileArgs) -> Result<(), failure::Error> {

    let dest_private_dir = args.dest_dir.join("private");
    fsutil::ensure_empty_dir(&dest_private_dir)?;

    let config = KinSettings::read(&project.config_file())?;
    let key_bytes = base64::decode(&config.master_key)?;
    let encryption_key = Key::from_slice(key_bytes.as_slice()).unwrap();
    let mut nonces = Vec::new();

    copy_dir_encrypted(&project.private_dir(), &dest_private_dir, &encryption_key, &mut nonces)?;

    Ok(())
}

fn copy_public_dir(project: &KinProject, dest_dir: &PathBuf) -> Result<(), failure::Error> {

    let dest_public_dir = dest_dir.join("public");
    fsutil::ensure_empty_dir(&dest_public_dir)?;

    copy_dir(&project.public_dir(), &dest_public_dir)?;

    Ok(())
}

fn copy_dir(source: &PathBuf, dest: &PathBuf) -> Result<(), failure::Error> {

    let contents = fs::read_dir(source)?;

    for item in contents {
        let item = item?;

        if item.metadata()?.is_dir() {
            let new_dest = dest.join(item.file_name());
            fs::create_dir(&new_dest)?;
            copy_dir(&item.path(), &new_dest)?;
        } else {
            let dest_path = dest.join(item.file_name());

            info!("copying {} to {}...",
                item.path().to_str().unwrap(),
                dest_path.to_str().unwrap());

            fs::copy(item.path(), dest_path)?;
        }
    }

    Ok(())
}

fn copy_dir_encrypted(source: &PathBuf,
    dest: &PathBuf,
    encryption_key: &Key,
    nonces: &mut Vec<NonceRecord>) -> Result<(), failure::Error> {

    let contents = fs::read_dir(source)?;

    for item in contents {
        let item = item?;

        if item.metadata()?.is_dir() {
            let new_dest = dest.join(item.file_name());
            fs::create_dir(&new_dest)?;
            copy_dir_encrypted(&item.path(), &new_dest, encryption_key, nonces)?;
        } else {

            let mut file_name = String::from(item.file_name().to_str().unwrap());
            file_name.push_str(".kin");

            let dest_path = dest.join(file_name);

            info!("copying {} to {}...",
                item.path().to_str().unwrap(),
                dest_path.to_str().unwrap());

            let nonce = secretbox::gen_nonce();
            let plaintext = fs::read(item.path())?;
            let encrypted = secretbox::seal(&plaintext, &nonce, &encryption_key);

            fs::write(dest_path, encrypted)?;

            nonces.push(NonceRecord {
                file: String::from(item.path().to_str().unwrap()),
                nonce: nonce
            });
        }
    }

    Ok(())
}

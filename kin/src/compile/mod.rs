mod readme;
mod zip;
use kin_core::{ BackupPackage, CompileArgs, EncryptedMasterKey, Error, KinProject, KinSettings };
use kin_core::{ bail, info, libsodium, fsutil };
use std::fs;
use std::fs::{ File, OpenOptions };
use std::iter::Iterator;
use std::path::PathBuf;
use self::zip::ZipWriter;

pub fn run(args: &CompileArgs) -> Result<(), Error> {

    let project = match &args.project_dir {
        Some(dir) => KinProject::from(&dir),
        None => KinProject::from(&std::env::current_dir()?)
    };

    let recip_name = &args.recipient;
    let settings = match project.settings() {
        Ok(settings) => settings,
        Err(e) => bail!("unable to read settings: {}", e)
    };

    let peers = settings.get_peers(recip_name)?;
    let master_key = match settings.master_key() {
        Ok(key) => key,
        Err(e) => bail!("invalid master key: {}", e)
    };

    let encrypted_keys: Vec<EncryptedMasterKey> = peers.iter()
        .map(|x| master_key.encrypt(&x.passphrase).unwrap())
        .collect();

    let dest_package = BackupPackage::init(&args.dest_dir, encrypted_keys)?;

    copy_public_dir(&project, &dest_package)?;
    copy_private_dir(&project, &dest_package)?;
    copy_decrypt_exes(&dest_package)?;
    copy_readmes(&project, &settings, &recip_name, &dest_package)?;

    Ok(())
}

fn copy_public_dir(src_project: &KinProject, dest_package: &BackupPackage) -> Result<(), Error> {

    let dest_archive_path = dest_package.public_archive_path();

    {
        let mut dest_archive = ZipWriter::new(&dest_archive_path)?;
        zip_dir(&src_project.public_dir(), &mut dest_archive, &PathBuf::new())?;
        dest_archive.finish()?;
    }

    platform::set_readonly(&dest_archive_path)?;

    Ok(())
}

fn copy_private_dir(src_project: &KinProject, dest_package: &BackupPackage) -> Result<(), Error> {

    if src_project.temp_file().exists() {
        fs::remove_file(src_project.temp_file())?;
    }

    let mut temp_archive = ZipWriter::new(&src_project.temp_file())?;
    zip_dir(&src_project.private_dir(), &mut temp_archive, &PathBuf::new())?;
    temp_archive.finish()?;

    let config = src_project.settings()?;
    let encryption_key = config.master_key()?;

    let dest_path = dest_package.private_archive_path();

    {
        let mut dest_file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&dest_path)?;

        let mut reader = File::open(&src_project.temp_file())?;
        libsodium::encrypt(&encryption_key, &mut reader, &mut dest_file)?;
    }

    platform::set_readonly(&dest_path)?;

    fs::remove_file(src_project.temp_file())?;

    Ok(())
}

fn zip_dir(source: &PathBuf, dest_archive: &mut ZipWriter, dest_dir: &PathBuf) -> Result<(), Error> {

    // TODO: Use walkdir crate instead. We already are... it's a third-party
    // dependency of one of our direct dependencies.
    // https://crates.io/crates/walkdir

    let contents = fs::read_dir(source)?;

    for item in contents {
        let item = item?;
        let metadata = match item.metadata() {
            Ok(m) => m,
            Err(e) => bail!("error reading metadata: {}", e)
        };

        if metadata.is_dir() {
            let dest_dir = dest_dir.join(item.file_name());
            dest_archive.add_dir(dest_dir.to_str().unwrap())?;
            zip_dir(&item.path(), dest_archive, &dest_dir)?;
        } else {
            let dest_path = dest_dir.join(item.file_name());
            let dest_path = dest_path.to_str().unwrap();

            info!("zipping {} to {}...",
                item.path().to_str().unwrap(),
                dest_path);

            dest_archive.add_file(&item.path(), dest_path)?;
        }
    }

    Ok(())
}

fn copy_decrypt_exes(dest_package: &BackupPackage) -> Result<(), Error> {

    fsutil::ensure_empty_dir(&dest_package.decrypt_exe_dir())?;

    let executable_archive_bytes = include_bytes!("decrypt_executables.zip").to_vec();
    zip::extract(&executable_archive_bytes, &dest_package.decrypt_exe_dir())?;

    Ok(())
}

fn copy_readmes(project: &KinProject, settings: &KinSettings, recipient: &String, dest_package: &BackupPackage) -> Result<(), Error> {

    let peers = settings.get_peers(&recipient)?.iter()
        .map(|p| readme::PeerModel {
            name: p.name.clone()
        })
        .collect();

    let recipient = settings.get_recipient(&recipient)
        .expect("could get peers, but no recipient");

    let model = readme::ReadmeModel {
        owner: settings.owner(),
        recipient: recipient.name.clone(),
        passphrase: recipient.passphrase.clone(),
        peers: peers
    };

    readme::render(&project.overview_readme_template(), &model, &dest_package.overview_readme_path())?;
    platform::set_readonly(&dest_package.overview_readme_path())?;

    readme::render(&project.decrypt_readme_template(), &model, &dest_package.decrypt_readme_path())?;
    platform::set_readonly(&dest_package.decrypt_readme_path())?;

    Ok(())
}

#[cfg(target_os = "linux")]
mod platform {
    use std::fs;
    use kin_core::Error;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    pub fn set_readonly(path: &PathBuf) -> Result<(), Error> {

        // Set read-only permissions for user, group, and others.
        let perms = PermissionsExt::from_mode(0o444);
        fs::set_permissions(path, perms)?;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use kin_core::Error;
    use std::path::PathBuf;

    pub fn set_readonly(_path: &PathBuf) -> Result<(), Error> {
        // TODO: Set readonly flag on file
        Ok(())
    }
}

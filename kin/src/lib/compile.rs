use super::backuppackage::BackupPackage;
use super::cmdline::CompileArgs;
use super::kinproject::KinProject;
use super::kinsettings::{ KinSettings };
use super::kinzip::KinZipWriter;
use super::libsodium::{ EncryptingWriter, EncryptedMasterKey };
use super::templating;
use super::templating::{ PeerModel, ReadmeModel };
use log::{ info };
use std::fs;
use std::fs::{ File, OpenOptions };
use std::iter::Iterator;
use std::path::PathBuf;

pub fn run(args: &CompileArgs) -> Result<(), failure::Error> {

    let project = match &args.project_dir {
        Some(dir) => KinProject::from(&dir),
        None => KinProject::from(&std::env::current_dir()?)
    };

    let recip_name = &args.recipient;
    let settings = project.settings()?;
    let peers = settings.get_peers(recip_name)?;
    let master_key = settings.master_key()?;
    let encrypted_keys: Vec<EncryptedMasterKey> = peers.iter()
        .map(|x| master_key.encrypt(&x.passphrase).unwrap())
        .collect();

    let dest_package = BackupPackage::init(&args.dest_dir, encrypted_keys)?;

    copy_public_dir(&project, &dest_package)?;
    copy_private_dir(&project, &dest_package)?;
    copy_exe(&dest_package)?;
    copy_readme(&project, &settings, &recip_name, &dest_package)?;

    Ok(())
}

fn copy_public_dir(src_project: &KinProject, dest_package: &BackupPackage) -> Result<(), failure::Error> {

    let dest_archive_path = dest_package.public_archive_path();
    let mut dest_archive = KinZipWriter::new(&dest_archive_path)?;

    zip_dir(&src_project.public_dir(), &mut dest_archive, &PathBuf::from("/"))?;

    dest_archive.finish()?;

    Ok(())
}

fn copy_private_dir(src_project: &KinProject, dest_package: &BackupPackage) -> Result<(), failure::Error> {

    if src_project.temp_file().exists() {
        fs::remove_file(src_project.temp_file())?;
    }

    let mut temp_archive = KinZipWriter::new(&src_project.temp_file())?;
    zip_dir(&src_project.private_dir(), &mut temp_archive, &PathBuf::from("/"))?;
    temp_archive.finish()?;

    let config = src_project.settings()?;
    let encryption_key = config.master_key()?;

    let dest_path = dest_package.private_archive_path();
    let mut dest_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dest_path)?;

    {
        let mut reader = File::open(&src_project.temp_file())?;
        let mut writer = EncryptingWriter::new(&encryption_key, &mut dest_file)?;

        let temp_archive_size = src_project.temp_file().metadata()?.len();
        writer.consume(&mut reader, temp_archive_size)?;
    }

    fs::remove_file(src_project.temp_file())?;

    Ok(())
}

fn zip_dir(source: &PathBuf, dest_archive: &mut KinZipWriter, dest_dir: &PathBuf) -> Result<(), failure::Error> {

    let contents = fs::read_dir(source)?;

    for item in contents {
        let item = item?;

        if item.metadata()?.is_dir() {
            let dest_dir = dest_dir.join(item.file_name());
            zip_dir(&item.path(), dest_archive, &dest_dir)?;
        } else {
            let dest_path = dest_dir.join(item.file_name());
            let dest_path = dest_path.to_str().unwrap();

            info!("zipping {} to {}...",
                item.path().to_str().unwrap(),
                dest_path);

            dest_archive.add_file(&item.path(), String::from(dest_path))?;
        }
    }

    Ok(())
}

fn copy_exe(dest_package: &BackupPackage) -> Result<(), failure::Error> {

    let src = std::env::current_exe()?;
    let dst = dest_package.decrypt_exe_path();
    fs::copy(src, dst)?;

    Ok(())
}

fn copy_readme(project: &KinProject, settings: &KinSettings, recipient: &String, dest_package: &BackupPackage) -> Result<(), failure::Error> {

    let peers = settings.get_peers(&recipient)?.iter()
        .map(|p| PeerModel {
            name: p.name.clone()
        })
        .collect();

    let recipient = settings.get_recipient(&recipient)?;

    let model = ReadmeModel {
        owner: settings.owner(),
        recipient: recipient.name.clone(),
        passphrase: recipient.passphrase.clone(),
        peers: peers
    };

    templating::render_readme(&project.template_readme(), &model, &dest_package.readme_path())?;

    Ok(())
}

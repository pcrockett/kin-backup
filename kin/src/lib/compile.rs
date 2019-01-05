use super::cmdline::CompileArgs;
use super::fsutil;
use super::kinproject::KinProject;
use super::kinsettings::KinSettings;
use super::kinzip::KinZipWriter;
use super::libsodium::{ EncryptingWriter, SymmetricKey };
use log::{ info };
use std::fs;
use std::fs::{ File, OpenOptions };
use std::path::PathBuf;

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
    let encryption_key = SymmetricKey::decode_base64(&config.master_key)?;

    copy_dir_encrypted(&project.private_dir(), &dest_private_dir, &encryption_key)?;

    Ok(())
}

fn copy_public_dir(project: &KinProject, dest_dir: &PathBuf) -> Result<(), failure::Error> {

    let dest_archive_path = dest_dir.join("public.zip");
    let mut dest_archive = KinZipWriter::new(&dest_archive_path)?;

    copy_dir(&project.public_dir(), &mut dest_archive, &PathBuf::from("/"))?;

    dest_archive.finish()?;

    Ok(())
}

fn copy_dir(source: &PathBuf, dest_archive: &mut KinZipWriter, dest_dir: &PathBuf) -> Result<(), failure::Error> {

    let contents = fs::read_dir(source)?;

    for item in contents {
        let item = item?;

        if item.metadata()?.is_dir() {
            let dest_dir = dest_dir.join(item.file_name());
            copy_dir(&item.path(), dest_archive, &dest_dir)?;
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

fn copy_dir_encrypted(source: &PathBuf,
    dest: &PathBuf,
    encryption_key: &SymmetricKey) -> Result<(), failure::Error> {

    let contents = fs::read_dir(source)?;

    for item in contents {
        let file = item?;

        if file.metadata()?.is_dir() {
            let new_dest = dest.join(file.file_name());
            fs::create_dir(&new_dest)?;
            copy_dir_encrypted(&file.path(), &new_dest, encryption_key)?;
        } else {

            let mut file_name = String::from(file.file_name().to_str().unwrap());
            file_name.push_str(".kin");

            let dest_path = dest.join(file_name);

            info!("copying {} to {}...",
                file.path().to_str().unwrap(),
                dest_path.to_str().unwrap());

            let mut dest_file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(dest_path)?;

            let mut reader = File::open(file.path())?;
            let mut writer = EncryptingWriter::new(encryption_key, &mut dest_file)?;

            writer.consume(&mut reader, file.metadata()?.len())?;
        }
    }

    Ok(())
}

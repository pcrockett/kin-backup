use super::cmdline::CompileArgs;
use super::fsutil;
use super::kinproject::KinProject;
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

fn copy_public_dir(project: &KinProject, dest_dir: &PathBuf) -> Result<(), failure::Error> {

    let dest_archive_path = dest_dir.join("public.zip");
    let mut dest_archive = KinZipWriter::new(&dest_archive_path)?;

    zip_dir(&project.public_dir(), &mut dest_archive, &PathBuf::from("/"))?;

    dest_archive.finish()?;

    Ok(())
}

fn copy_private_dir(project: &KinProject, args: &CompileArgs) -> Result<(), failure::Error> {

    if project.temp_file().exists() {
        fs::remove_file(project.temp_file())?;
    }

    let mut temp_archive = KinZipWriter::new(&project.temp_file())?;
    zip_dir(&project.private_dir(), &mut temp_archive, &PathBuf::from("/"))?;
    temp_archive.finish()?;

    let config = project.settings()?;
    let encryption_key = SymmetricKey::decode_base64(&config.master_key)?;

    let dest_path = args.dest_dir.join("private.kin");
    let mut dest_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dest_path)?;

    {
        let mut reader = File::open(&project.temp_file())?;
        let mut writer = EncryptingWriter::new(&encryption_key, &mut dest_file)?;

        let temp_archive_size = project.temp_file().metadata()?.len();
        writer.consume(&mut reader, temp_archive_size)?;
    }

    fs::remove_file(project.temp_file())?;

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

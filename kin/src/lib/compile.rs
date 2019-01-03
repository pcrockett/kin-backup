use super::cmdline::CompileArgs;
use super::fsutil;
use super::kinproject::KinProject;
use log::{ info };
use std::fs;
use std::path::PathBuf;

pub fn run(args: CompileArgs) -> Result<(), failure::Error> {

    fsutil::ensure_empty_dir(&args.dest_dir)?;
    let project = KinProject::from(args.project_dir);

    copy_public_dir(&project, args.dest_dir)?;

    Ok(())
}

fn copy_public_dir(project: &KinProject, dest_dir: PathBuf) -> Result<(), failure::Error> {

    let dest_public_dir = dest_dir.join("public");
    fsutil::ensure_empty_dir(&dest_public_dir)?;

    copy_dir(project.public_dir(), dest_public_dir)?;

    Ok(())
}

fn copy_dir(source: PathBuf, dest: PathBuf) -> Result<(), failure::Error> {

    let contents = fs::read_dir(source)?;

    for item in contents {
        let item = item?;

        if item.metadata()?.is_dir() {
            let new_dest = dest.join(item.file_name());
            fs::create_dir(&new_dest)?;
            copy_dir(item.path(), new_dest)?;
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

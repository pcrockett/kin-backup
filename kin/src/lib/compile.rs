use super::cmdline::CompileArgs;
use super::fsutil;
use super::kinproject::KinProject;
use std::path::Path;

pub fn run(args: CompileArgs) -> Result<(), failure::Error> {

    fsutil::ensure_empty_dir(&args.dest_dir)?;
    let project = KinProject::from(args.project_dir);

    copy_public_dir(&project, &args.dest_dir)?;

    Ok(())
}

fn copy_public_dir(project: &KinProject, dest_dir: &Path) -> Result<(), failure::Error> {

    let dest_public_dir = dest_dir.join("public");
    fsutil::ensure_empty_dir(&dest_public_dir)?;

    let public_contents = std::fs::read_dir(project.public_dir())?;

    for item in public_contents {
        let item = item?;

        if item.metadata()?.is_dir() {
            // TODO: Copy this subdir
        } else {
            let dest_path = dest_public_dir.join(item.file_name());
            std::fs::copy(item.path(), dest_path)?;
        }
    }

    Ok(())
}

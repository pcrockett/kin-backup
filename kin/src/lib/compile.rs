use super::cmdline::CompileArgs;
use super::fsutil;
use super::kinproject::KinProject;

pub fn run(args: CompileArgs) -> Result<(), failure::Error> {

    fsutil::ensure_empty_dir(&args.dest_dir)?;
    let project = KinProject::from(args.project_dir);

    let dest_dir = args.dest_dir.to_str().unwrap();
    Err(failure::err_msg(
        format!("Not implemented yet, but the project dir is {} and destination is {}",
        project.path().to_str().unwrap(),
        dest_dir)
    ))
}

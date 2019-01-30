use super::cmdline::DecryptArgs;
use log:: { info };

pub fn run(args: &DecryptArgs) -> Result<(), failure::Error> {

    let dest_dir = match &args.dest_dir {
        Some(dir) => dir.to_owned(),
        None => panic!("TODO: Prompt user for a destination directory")
    };

    let source_dir = match &args.backup_dir {
        Some(dir) => dir.to_owned(),
        None => std::env::current_dir()?
    };

    info!("source_dir: {}", source_dir.to_str().unwrap());
    info!("dest_dir: {}", dest_dir.to_str().unwrap());

    Ok(())
}

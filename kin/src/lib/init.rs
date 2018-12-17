use super::cmdline;
use log::{ info };
use std::fs;
use std::io;
use std::path::Path;

pub fn run(args: cmdline::InitArgs) -> Result<(), failure::Error> {

    match args.directory {
        Some(dir) => ensure_dir(&dir)?,
        None => return Err(failure::err_msg("TODO: Figure out current directory"))
    }

    Ok(())
}

fn ensure_dir(path: &Path) -> io::Result<()> {

    if path.exists() {

        let metadata = fs::metadata(path)?;
        if metadata.is_dir() {
            info!("directory {} already exists", path.to_str().unwrap());
            return Ok(());
        }

        return Err(
            io::Error::new(
                io::ErrorKind::AlreadyExists, 
                format!("{} already exists", path.to_str().unwrap())
            )
        );
    }

    fs::create_dir(path)?;
    info!("created {}", path.to_str().unwrap());
    Ok(())
}

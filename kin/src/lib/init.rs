use super::cmdline;
use log::{ info };
use std::fs;
use std::io;
use std::path::Path;

pub fn run(args: cmdline::InitArgs) -> Result<(), failure::Error> {

    let project_dir = match args.directory {
        Some(dir) => {

            let result = ensure_dir(&dir);

            if result.is_ok() {
                Ok(dir)
            } else {
                Err(result.unwrap_err())
            }
        },
        None => std::env::current_dir()
    }?;

    info!("{}", project_dir.to_str().unwrap());

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

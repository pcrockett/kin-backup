use log::{ info };
use std::fs;
use std::io;
use std::path::Path;

pub fn ensure_empty_dir(path: &Path) -> io::Result<()> {

    if !path.exists() {
        fs::create_dir(path)?;
        info!("created {}", path.to_str().unwrap());
        return Ok(());
    }

    let metadata = fs::metadata(path)?;
    if !metadata.is_dir() {
        return Err(
            io::Error::new(
                io::ErrorKind::AlreadyExists, 
                format!("{} is not a directory", path.to_str().unwrap())
            )
        );
    }

    let is_not_empty = fs::read_dir(&path)?.any(|_| true);
    if is_not_empty {
        return Err(
            io::Error::new(
                io::ErrorKind::Other,
                format!("{} is not empty", path.to_str().unwrap())
            )
        );
    }

    info!("directory {} already exists and is empty", path.to_str().unwrap());
    Ok(())
}
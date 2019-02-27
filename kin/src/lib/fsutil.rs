use failure::{ bail };
use std::fs;
use std::path::Path;

pub fn ensure_empty_dir(path: &Path) -> Result<(), failure::Error> {

    if !path.exists() {
        match fs::create_dir(path) {
            Ok(x) => return Ok(x),
            Err(e) => bail!("unable to create {}: {}", path.to_str().unwrap(), e)
        };
    }

    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => bail!("unable to get metadata for {}: {}", path.to_str().unwrap(), e)
    };

    if !metadata.is_dir() {
        bail!("{} is not a directory", path.to_str().unwrap());
    }

    let is_not_empty = match fs::read_dir(&path) {
        Ok(mut contents) => contents.any(|_| true),
        Err(e) => bail!("unable to list contents of {}: {}", path.to_str().unwrap(), e)
    };

    if is_not_empty {
        bail!("{} is not empty", path.to_str().unwrap());
    }

    Ok(())
}
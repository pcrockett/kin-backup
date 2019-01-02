use super::fsutil;
use std::path::PathBuf;

pub struct KinProject {
    path: PathBuf
}

impl KinProject {

    pub fn from(path: Option<PathBuf>) -> KinProject {

        if path.is_some() {

            return KinProject {
                path: path.unwrap()
            };

        } else {

            return KinProject {
                path: std::env::current_dir().unwrap()
            };
        }
    }

    pub fn init(path: Option<PathBuf>) -> Result<KinProject, failure::Error> {

        if path.is_some() {

            let dir = path.unwrap();
            fsutil::ensure_empty_dir(&dir)?;

            return Ok(
                KinProject {
                    path: dir
                }
            );

        } else {

            let dir = std::env::current_dir()?;
            fsutil::ensure_empty_dir(&dir)?;

            return Ok(
                KinProject {
                    path: dir
                }
            );
        }
    }

    pub fn path(self: &KinProject) -> &PathBuf {
        &self.path
    }
}

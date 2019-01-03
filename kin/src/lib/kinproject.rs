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

        let dir: PathBuf;

        if path.is_some() {

            dir = path.unwrap();
            fsutil::ensure_empty_dir(&dir)?;

        } else {

            dir = std::env::current_dir()?;
            fsutil::ensure_empty_dir(&dir)?;
        }

        let project = KinProject {
            path: dir
        };

        let subdirs = [
            project.public_dir(),
            project.private_dir(),
            project.config_dir()
        ];

        for subdir in subdirs.iter() {
            fsutil::ensure_empty_dir(&subdir)?;
        }

        Ok(project)
    }

    pub fn public_dir(&self) -> PathBuf {
        self.path.join("public")
    }

    pub fn private_dir(&self) -> PathBuf {
        self.path.join("private")
    }

    pub fn config_dir(&self) -> PathBuf {
        self.path.join(".kin")
    }
}

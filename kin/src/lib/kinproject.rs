use super::fsutil;
use std::path::PathBuf;

pub struct KinProject {
    path: PathBuf
}

impl KinProject {

    pub fn from(path: &PathBuf) -> KinProject {

        KinProject {
            path: path.to_owned()
        }
    }

    pub fn init(path: &PathBuf) -> Result<KinProject, failure::Error> {

        fsutil::ensure_empty_dir(path)?;

        let project = KinProject {
            path: path.to_owned()
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

    pub fn config_file(&self) -> PathBuf {
        self.config_dir().join("config.json")
    }

    pub fn temp_file(&self) -> PathBuf {
        self.config_dir().join("temp")
    }
}

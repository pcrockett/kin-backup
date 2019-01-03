use std::fs::File;
use std::io::{ BufWriter, Write };
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct KinRecipient {
    pub name: String,
    pub password: String
}

#[derive(Serialize, Deserialize)]
pub struct KinSettings {
    pub master_key: String,
    pub recipients: Vec<KinRecipient>
}

impl KinSettings {

    pub fn write(&self, path: &PathBuf) -> Result<(), failure::Error> {

        let config_serialized = serde_json::to_string_pretty(self)?;

        let file = File::create(path)?;
        let mut file = BufWriter::new(file);
        file.write(config_serialized.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    pub fn read(path: &PathBuf) -> Result<KinSettings, failure::Error> {

        let file = File::open(path)?;
        let settings = serde_json::from_reader(file)?;
        Ok(settings)
    }
}
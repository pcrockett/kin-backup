use super::libsodium::MasterKey;
use failure::{ bail };
use std::fs::File;
use std::io::{ BufWriter, Write };
use std::iter::Iterator;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct KinRecipient {
    pub name: String,
    pub passphrase: String
}

#[derive(Serialize, Deserialize)]
pub struct KinSettings {
    owner: String,
    master_key: String,
    pub recipients: Vec<KinRecipient>
}

impl KinSettings {

    pub fn new(owner: &String, recipients: Vec<KinRecipient>) -> KinSettings {
        KinSettings {
            owner: owner.clone(),
            master_key: MasterKey::new().encode_base64(),
            recipients: recipients
        }
    }

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

    pub fn get_recipient(&self, name: &String) -> Result<&KinRecipient, failure::Error> {

        let recip: Vec<&KinRecipient> = self.recipients.iter()
            .filter(|x| &x.name == name)
            .collect();

        if recip.len() != 1 {
            bail!("{} recipients found with the name \"{}\"", recip.len(), name);
        }

        Ok(recip[0])
    }

    pub fn get_peers(&self, name: &String) -> Result<Vec<&KinRecipient>, failure::Error> {

        let recip = self.get_recipient(name)?; // Makes sure this is a valid recipient

        let others: Vec<&KinRecipient> = self.recipients.iter()
            .filter(|x| &x.name != &recip.name)
            .collect();

        Ok(others)
    }

    pub fn master_key(&self) -> Result<MasterKey, failure::Error> {
        MasterKey::decode_base64(&self.master_key)
    }
}
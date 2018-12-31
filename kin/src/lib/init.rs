use super::cmdline;
use failure::{ bail };
use log::{ info };
use sodiumoxide::crypto::stream;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct KinRecipient<'a> {
    name: &'a str,
    password: &'a str
}

#[derive(Serialize, Deserialize)]
struct KinSettings<'a> {
    master_key: &'a str,
    recipients: Vec<KinRecipient<'a>>
}

pub fn run(args: cmdline::InitArgs) -> Result<(), failure::Error> {

    let project_dir = get_project_dir(args.directory)?;
    let is_not_empty = fs::read_dir(&project_dir)?.any(|_| true);

    if is_not_empty {
        bail!("{} is not empty.", project_dir.to_str().unwrap());
    }

    let subdirs = [
            "public",
            "secret",
            ".kin"
        ].iter()
        .map(|x| project_dir.join(x));

    for subdir in subdirs {
        ensure_dir(&subdir)?;
    }

    let file = fs::File::create(project_dir.join(".kin/config.json"))?;
    let mut file = io::BufWriter::new(file);
    let key = stream::gen_key();
    let key_base64 = &base64::encode(&key[..]);

    let recipients: Vec<KinRecipient> = args.recipients.iter().map(|r| KinRecipient {
        name: r,
        password: "not implemented yet"
    }).collect();

    let config = KinSettings {
        master_key: key_base64,
        recipients: recipients
    };

    let config_serialized = serde_json::to_string_pretty(&config)?;

    file.write(config_serialized.as_bytes())?;
    file.flush()?;

    Ok(())
}

fn get_project_dir(path: Option<PathBuf>) -> Result<PathBuf, failure::Error> {

    if path.is_some() {
        let dir = path.unwrap();
        ensure_dir(&dir)?;
        return Ok(dir);
    } else {
        return Ok(std::env::current_dir()?);
    }
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

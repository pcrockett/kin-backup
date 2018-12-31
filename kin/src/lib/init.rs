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
struct KinRecipient {
    name: String,
    password: String
}

#[derive(Serialize, Deserialize)]
struct KinSettings {
    master_key: String,
    recipients: Vec<KinRecipient>
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
        name: r.to_owned(),
        password: random_password()
    }).collect();

    let config = KinSettings {
        master_key: key_base64.to_owned(),
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

fn random_password<'a>() -> String {
    let word_list = get_words();
    random_password_from(word_list)
}

fn random_password_from(word_list: Vec<&str>) -> String {

    let mut password = String::new();
    (0..6).map(|_| random_int() as usize)
        .map(|r| r % word_list.len())
        .map(|i| word_list[i])
        .for_each(|word| {
            password.push_str(word);
            password.push(' ');
        });

    // Remove extra last space
    password.pop();

    let password = password; // Get immutable value for returning
    password
}

fn random_int() -> u32 {

    let buffer: &mut [u8; std::mem::size_of::<u32>()] = &mut [0; std::mem::size_of::<u32>()];
    sodiumoxide::randombytes::randombytes_into(buffer);

    let buffer = *buffer; // Get immutable array

    unsafe {
        std::mem::transmute::<[u8; 4], u32>(buffer)
    }
}

fn get_words() -> Vec<&'static str> {

    let raw_file = include_str!("eff_large_wordlist.txt");

    raw_file.split_whitespace()
        .collect()
}

#[cfg(test)]
mod tests {

    #[test]
    fn word_list() {
        let words = super::get_words();

        assert_eq!(words[0], "abacus");
        assert_eq!(words[words.len() - 1], "zoom");
    }

    #[test]
    fn random_password_single_word() {
        let words = vec!("foo");
        let password = super::random_password_from(words);

        assert_eq!(password, "foo foo foo foo foo foo");
    }
}

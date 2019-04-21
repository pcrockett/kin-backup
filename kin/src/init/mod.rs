use kin_core::libsodium;
use kin_core::ui;
use kin_core::{Error, InitArgs, KinProject, KinRecipient, KinSettings};
use std::fs::File;
use std::io::Write;

pub fn run(args: &InitArgs) -> Result<(), Error> {
    let project = match &args.directory {
        Some(dir) => KinProject::init(&dir)?,
        None => KinProject::init(&std::env::current_dir()?)?,
    };

    let owner = match &args.owner {
        Some(name) => name.clone(),
        None => prompt_owner_name()?,
    };

    let recipients: Vec<KinRecipient> = args
        .recipients
        .iter()
        .map(|r| KinRecipient {
            name: r.to_owned(),
            passphrase: random_passphrase(),
        })
        .collect();

    let config = KinSettings::new(&owner, recipients);
    config.write(&project.config_file())?;

    let overview_contents = include_bytes!("readme_templates/overview.md");
    let mut file = File::create(project.overview_readme_template())?;
    file.write_all(overview_contents)?;

    let decrypt_contents = include_bytes!("readme_templates/decrypt.md");
    let mut file = File::create(project.decrypt_readme_template())?;
    file.write_all(decrypt_contents)?;

    Ok(())
}

fn prompt_owner_name() -> Result<String, Error> {
    loop {
        let input = ui::prompt("Enter your name: ")?;
        if input.len() == 0 {
            continue; // TODO: Allow owner name to be optional?
        }

        return Ok(input);
    }
}

fn random_passphrase() -> String {
    let word_list = get_words();
    random_passphrase_from(word_list)
}

fn random_passphrase_from(word_list: Vec<&str>) -> String {
    let mut passphrase = String::new();
    (0..10)
        .map(|_| random_int() as usize)
        .map(|r| r % word_list.len())
        .map(|i| word_list[i])
        .for_each(|word| {
            passphrase.push_str(word);
            passphrase.push(' ');
        });

    // Remove extra last space
    passphrase.pop();

    let passphrase = passphrase; // Get immutable value for returning
    passphrase
}

fn random_int() -> u32 {
    let buffer: &mut [u8; std::mem::size_of::<u32>()] = &mut [0; std::mem::size_of::<u32>()];
    libsodium::randombytes_into(buffer);

    let buffer = *buffer; // Get immutable array

    unsafe { std::mem::transmute::<[u8; 4], u32>(buffer) }
}

fn get_words() -> Vec<&'static str> {
    // Got this word list from:
    // https://www.eff.org/deeplinks/2016/07/new-wordlists-random-passphrases

    let raw_file = include_str!("eff_large_wordlist.txt");

    raw_file.split_whitespace().collect()
}

#[cfg(test)]
mod tests {

    #[test]
    fn word_list() {
        let words = super::get_words();

        assert_eq!(words[0], "abacus");
        assert_eq!(words[words.len() - 1], "zoom");

        assert!(words.len() >= 7776, "Number of words has decreased. Either use a larger word list or consider increasing the number of words in passphrases to maintain a high entropy.");
    }

    #[test]
    fn random_passphrase_single_word() {
        let words = vec!["foo"];
        let passphrase = super::random_passphrase_from(words);

        assert_eq!(passphrase, "foo foo foo foo foo foo foo foo foo foo");
    }
}

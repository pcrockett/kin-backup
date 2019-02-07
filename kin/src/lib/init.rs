use super::cmdline::InitArgs;
use super::kinproject::KinProject;
use super::kinsettings::{ KinSettings, KinRecipient };
use super::libsodium;
use std::fs::File;
use std::io::{ BufWriter, Write };

pub fn run(args: &InitArgs) -> Result<(), failure::Error> {

    let project = match &args.directory {
        Some(dir) => KinProject::init(&dir)?,
        None => KinProject::init(&std::env::current_dir()?)?
    };

    let recipients: Vec<KinRecipient> = args.recipients.iter().map(|r| KinRecipient {
        name: r.to_owned(),
        passphrase: random_passphrase()
    }).collect();

    let config = KinSettings::new(recipients);
    config.write(&project.config_file())?;

    let template_contents = include_bytes!("readme-template.md");
    let file = File::create(project.template_readme())?;
    let mut file = BufWriter::new(file);
    file.write_all(template_contents)?;

    Ok(())
}

fn random_passphrase() -> String {
    let word_list = get_words();
    random_passphrase_from(word_list)
}

fn random_passphrase_from(word_list: Vec<&str>) -> String {

    let mut passphrase = String::new();
    (0..10).map(|_| random_int() as usize)
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

    unsafe {
        std::mem::transmute::<[u8; 4], u32>(buffer)
    }
}

fn get_words() -> Vec<&'static str> {

    // Got this word list from:
    // https://www.eff.org/deeplinks/2016/07/new-wordlists-random-passphrases

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

        assert!(words.len() >= 7776, "Number of words has decreased. Either use a larger word list or consider increasing the number of words in passphrases to maintain a high entropy.");
    }

    #[test]
    fn random_passphrase_single_word() {
        let words = vec!("foo");
        let passphrase = super::random_passphrase_from(words);

        assert_eq!(passphrase, "foo foo foo foo foo foo foo foo foo foo");
    }
}

use super::cmdline::InitArgs;
use super::kinproject::KinProject;
use super::kinsettings::{ KinSettings, KinRecipient };
use super::libsodium;

pub fn run(args: &InitArgs) -> Result<(), failure::Error> {

    let project = match &args.directory {
        Some(dir) => KinProject::init(&dir)?,
        None => KinProject::init(&std::env::current_dir()?)?
    };

    let recipients: Vec<KinRecipient> = args.recipients.iter().map(|r| KinRecipient {
        name: r.to_owned(),
        password: random_password()
    }).collect();

    let config = KinSettings::new(recipients);
    config.write(&project.config_file())?;

    Ok(())
}

fn random_password() -> String {
    let word_list = get_words();
    random_password_from(word_list)
}

fn random_password_from(word_list: Vec<&str>) -> String {

    let mut password = String::new();
    (0..10).map(|_| random_int() as usize)
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

        assert!(words.len() >= 7776, "Number of words has decreased. Either use a larger word list or consider increasing the number of words in passwords to maintain a high entropy.");
    }

    #[test]
    fn random_password_single_word() {
        let words = vec!("foo");
        let password = super::random_password_from(words);

        assert_eq!(password, "foo foo foo foo foo foo foo foo foo foo");
    }
}

use structopt::StructOpt;
pub use quicli::prelude::*;

#[derive(StructOpt)]
#[structopt(name = "kin", about = "Easy, secure backups for your next of kin")]
pub enum CliArgs {

    /// Start a backup project
    #[structopt(name = "init")]
    Init {

        /// The directory where you will assemble the things you need to back up
        #[structopt(parse(from_os_str))]
        directory: Option<std::path::PathBuf>
    },

    /// Compile a backup
    #[structopt(name = "compile")]
    Compile {

        /// The destination directory where you want to generate the backup
        #[structopt(name = "destination", parse(from_os_str))]
        dest_dir: std::path::PathBuf
    }
}

pub fn parse() -> CliArgs {
    CliArgs::from_args()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn init_no_dir() {
        let args = [ "kin", "init" ].iter();

        let parsed = CliArgs::from_iter(args);
        match parsed {
            CliArgs::Init { directory } => assert_eq!(directory, Option::None),
            _ => panic!("not an init subcommand")
        };
    }

    #[test]
    fn init_with_dir() {

        let args = [ "kin", "init", "foo/bar" ].iter();

        let parsed = CliArgs::from_iter(args);
        match parsed {
            CliArgs::Init { directory } => {
                match directory {
                    Some(path) => assert_eq!("foo/bar", path.to_str().unwrap()),
                    None => panic!("no directory specified")
                }
            }
            _ => panic!("not an init subcommand")
        };

    }
}
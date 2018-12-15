use structopt::StructOpt;
pub use quicli::prelude::*;

#[derive(StructOpt)]
#[structopt(name = "kin", about = "Easy, secure backups for your next of kin")]
pub enum CliArgs {

    /// Start a backup project
    #[structopt(name = "init")]
    Init(InitArgs),

    /// Compile a backup
    #[structopt(name = "compile")]
    Compile(CompileArgs)
}

#[derive(StructOpt)]
pub struct InitArgs {
    /// The directory where you will assemble the things you need to back up
    #[structopt(parse(from_os_str))]
    pub directory: Option<std::path::PathBuf>
}

#[derive(StructOpt)]
pub struct CompileArgs {
    /// The destination directory where you want to generate the backup
    #[structopt(name = "destination", parse(from_os_str))]
    #[allow(dead_code)]
    dest_dir: std::path::PathBuf
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
        let init_args = match parsed {
            CliArgs::Init(args) => args,
            _ => panic!("not an init subcommand")
        };

        assert_eq!(init_args.directory, Option::None)
    }

    #[test]
    fn init_with_dir() {

        let args = [ "kin", "init", "foo/bar" ].iter();

        let parsed = CliArgs::from_iter(args);
        let init_args = match parsed {
            CliArgs::Init(args) => args,
            _ => panic!("not an init subcommand")
        };

        match init_args.directory {
            Some(path) => assert_eq!("foo/bar", path.to_str().unwrap()),
            None => panic!("no directory specified")
        };
    }
}
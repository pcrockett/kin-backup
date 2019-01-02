use structopt::StructOpt;
pub use quicli::prelude::*;

#[derive(StructOpt)]
#[structopt(name = "kin", about = "Easy, secure backups for your next of kin")]
struct CliArgs {

    #[structopt(subcommand)]
    pub cmd: SubCommand,

    #[structopt(flatten)]
    verbosity: Verbosity
}

#[derive(StructOpt)]
pub enum SubCommand {

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
    pub directory: Option<std::path::PathBuf>,

    /// Specify to whom you plan to give your backups
    #[structopt(short = "r", long = "recipients")]
    pub recipients: Vec<String>
}

#[derive(StructOpt)]
pub struct CompileArgs {
    /// The destination directory where you want to generate the backup
    #[structopt(name = "destination", parse(from_os_str))]
    pub dest_dir: std::path::PathBuf
}

pub fn parse() -> SubCommand {
    let args = CliArgs::from_args();
    args.verbosity.setup_env_logger("kin")
        .expect("unable to setup env logger");

    args.cmd
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn init_no_dir() {
        let args = [ "kin", "init" ].iter();

        let parsed = CliArgs::from_iter(args);
        let init_args = match parsed.cmd {
            SubCommand::Init(args) => args,
            _ => panic!("not an init subcommand")
        };

        assert_eq!(init_args.directory, Option::None)
    }

    #[test]
    fn init_with_dir() {

        let args = [ "kin", "init", "foo/bar" ].iter();

        let parsed = CliArgs::from_iter(args);
        let init_args = match parsed.cmd {
            SubCommand::Init(args) => args,
            _ => panic!("not an init subcommand")
        };

        match init_args.directory {
            Some(path) => assert_eq!("foo/bar", path.to_str().unwrap()),
            None => panic!("no directory specified")
        };
    }

    #[test]
    fn init_with_recipients() {

        let args = [
            "kin", "init", "--recipients", "foo@bar.com", "hi@bye.com"
        ].iter();

        let parsed = CliArgs::from_iter(args);
        let init_args = match parsed.cmd {
            SubCommand::Init(args) => args,
            _ => panic!("not an init subcommand")
        };

        assert_eq!(init_args.recipients.len(), 2);
        assert_eq!(init_args.recipients[0], "foo@bar.com");
        assert_eq!(init_args.recipients[1], "hi@bye.com");
    }

    #[test]
    fn compile_with_destination() {

        let args = [
            "kin", "compile", "~/temp"
        ].iter();

        let parsed = CliArgs::from_iter(args);
        let compile_args = match parsed.cmd {
            SubCommand::Compile(args) => args,
            _ => panic!("not a compile subcommand")
        };

        let destination = compile_args.dest_dir.to_str().unwrap();
        assert_eq!(destination, "~/temp");
    }
}
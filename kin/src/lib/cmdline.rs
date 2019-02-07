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
    Compile(CompileArgs),

    /// Decrypt a backup
    #[structopt(name = "decrypt")]
    Decrypt(DecryptArgs)
}

#[derive(StructOpt)]
pub struct InitArgs {
    /// The directory where you will assemble the things you need to back up
    #[structopt(parse(from_os_str))]
    pub directory: Option<std::path::PathBuf>,

    /// Specify to whom you plan to give your backups
    #[structopt(short = "r", long = "recipients")]
    pub recipients: Vec<String>,

    /// Specify your name (for readme that gets distributed to backup holders)
    #[structopt(short = "o", long = "owner")]
    pub owner: Option<String>
}

#[derive(StructOpt)]
pub struct CompileArgs {
    /// The destination directory where you want to generate the backup
    #[structopt(name = "dest-dir", parse(from_os_str))]
    pub dest_dir: std::path::PathBuf,

    /// The recipient for whom you're compiling your backup
    #[structopt(short = "r", long = "recipient")]
    pub recipient: String,

    /// The project directory
    #[structopt(short = "p", long = "project-dir", parse(from_os_str))]
    pub project_dir: Option<std::path::PathBuf>
}

#[derive(StructOpt)]
pub struct DecryptArgs {

    /// The directory containing the backup data
    #[structopt(short = "b", long = "backup-dir", parse(from_os_str))]
    pub backup_dir: Option<std::path::PathBuf>,

    /// The destination decrypted archive path
    #[structopt(short = "d", long = "destination", parse(from_os_str))]
    pub destination: Option<std::path::PathBuf>
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
    fn init_with_owner() {

        let args = [
            "kin", "init", "--owner", "chuck"
        ].iter();

        let parsed = CliArgs::from_iter(args);
        let init_args = match parsed.cmd {
            SubCommand::Init(args) => args,
            _ => panic!("not an init subcommand")
        };

        assert_eq!(init_args.owner.unwrap(), "chuck");
    }

    #[test]
    fn compile_with_destination() {

        let args = [
            "kin", "compile", "~/temp", "--recipient", "foo@bar.baz"
        ].iter();

        let parsed = CliArgs::from_iter(args);
        let compile_args = match parsed.cmd {
            SubCommand::Compile(args) => args,
            _ => panic!("not a compile subcommand")
        };

        let destination = compile_args.dest_dir.to_str().unwrap();
        assert_eq!(destination, "~/temp");
    }

    #[test]
    fn compile_with_proj_dir() {
        let args = [
            "kin", "compile", "~/temp",
            "--recipient", "foo@bar.baz",
            "--project-dir", "~/foo/bar"
        ].iter();

        let parsed = CliArgs::from_iter(args);
        let compile_args = match parsed.cmd {
            SubCommand::Compile(args) => args,
            _ => panic!("not a compile subcommand")
        };

        let proj_dir = match compile_args.project_dir {
            Some(dir) => dir,
            _ => panic!("no project dir specified")
        };

        assert_eq!(proj_dir.to_str().unwrap(), "~/foo/bar");
    }

    #[test]
    /// If no parameters are specified, that's ok; prompt the user for
    /// information. This should be the most user-friendly portion of the
    /// software.
    fn decrypt_no_parameters() {
        let args = [
            "kin", "decrypt"
        ].iter();

        let parsed = CliArgs::from_iter(args);
        let decrypt_command = match parsed.cmd {
            SubCommand::Decrypt(args) => args,
            _ => panic!("not a decrypt subcommand")
        };

        assert_eq!(decrypt_command.backup_dir, None);
        assert_eq!(decrypt_command.destination, None);
    }

    #[test]
    fn decrypt_with_parameters() {
        let args = [
            "kin", "decrypt",
            "--backup-dir", "~/foo",
            "--destination", "~/bar"
        ].iter();

        let parsed = CliArgs::from_iter(args);
        let decrypt_command = match parsed.cmd {
            SubCommand::Decrypt(args) => args,
            _ => panic!("not a decrypt subcommand")
        };

        let backup_dir = decrypt_command.backup_dir.unwrap();
        let destination = decrypt_command.destination.unwrap();

        assert_eq!(backup_dir.to_str().unwrap(), "~/foo");
        assert_eq!(destination.to_str().unwrap(), "~/bar");
    }
}